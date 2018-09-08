//! Emulates transaction execution and allows for real-time debugging
//! Debugs one transaction at a time (1:1 One VM, One TX)
use log::{info, error, warn, log};
use sputnikvm::{ValidTransaction, HeaderParams, SeqTransactionVM, AccountChange, errors::{RequireError, CommitError}, AccountCommitment, VM};
use futures::future::Future;
use sputnikvm_network_foundation::ByzantiumPatch;
use failure::Error;
use web3::{
    api::Web3,
    Transport,
    types::{BlockNumber, U256, Bytes},
};
use std::rc::Rc;

use super::{
    err::EmulError,
};


/// An action or what should happen for the next step of execution
pub enum Action {
    /// step back one instruction
    StepBack,
    /// step forward one instruction
    StepForward,
    /// RunUntil a PC
    RunUntil(usize),
    /// finish instruction
    Finish,
    /// execute to the end
    Exec,
}


/// Emulation Object
pub struct Emulator<T: Transport> {
    vm: SeqTransactionVM<ByzantiumPatch>,
    positions: Vec<usize>,
    transaction: (ValidTransaction, HeaderParams),
    client: web3::Web3<T>,
}

/// a vm that emulates a transaction, allowing for mutations during execution
impl<T> Emulator<T> where T: Transport {
    /// Create a new Emulator
    pub fn new(transaction: ValidTransaction, header: HeaderParams, client: Web3<T>) -> Self {
        Emulator {
            transaction: (transaction.clone(), header.clone()),
            vm: sputnikvm::TransactionVM::new(transaction, header),
            positions: Vec::new(),
            client
        }
    }
    
    /// fire the vm, with the specified Action
    pub fn fire(&mut self, action: Action) -> Result<(), EmulError> {
        match action {
            Action::StepBack => self.step_back(),
            Action::StepForward => self.step_forward(),
            Action::RunUntil(pc) => self.run_until(pc),
            Action::Exec => self.run(),
            Action::Finish => self.run()
        }
    }

    pub fn output(&self) -> Vec<u8> {
        self.vm.out().into()
    }

    fn new_vm(&self) -> Result<SeqTransactionVM<ByzantiumPatch>, EmulError> {
        let (txinfo, header) = self.transaction.clone();
        let mut new_vm = sputnikvm::TransactionVM::new(txinfo, header);

        self.vm.accounts().map(|acc| {
            // only commit accounts to new VM which have been previously committed
            match acc {
                AccountChange::Full{nonce, address, balance, code, ..} => {
                    info!("Committing account {:#x} from previous vm", address);
                    new_vm.commit_account(AccountCommitment::Full {
                        nonce: nonce.clone(), 
                        address: address.clone(), 
                        balance: balance.clone(), 
                        code: code.clone(),
                    })?;
                    Ok(())
                }
                AccountChange::Nonexist(addr) => new_vm.commit_account(AccountCommitment::Nonexist(addr.clone())),
                _=> {
                    warn!("Account Commitment could not be made!");
                    Ok(())
                }
            }
        }).collect::<Result<(), CommitError>>()?;
        Ok(new_vm)
    }

    fn step_back(&mut self) -> Result<(), EmulError> {
        let mut last_pos = 0;
        if let Some(x) = self.positions.pop() {
            last_pos = x;
        } else { // if nothing is on the positions stack, return a new VM with any accounts that may have been commmitted
            let new_vm = self.new_vm()?;
            std::mem::replace(&mut self.vm, new_vm);
            return Ok(());
        }
        self.positions.clear();
        let mut pos_goal = 0;
        let new_vm = self.new_vm()?; 
        
        info!("Pos: {}", last_pos);
        while pos_goal < last_pos {
            step(&mut self.vm, &self.client)?;
            if let Some(x) = self.vm.current_state() {
                pos_goal = x.position;
                self.positions.push(x.position);
            } else {
                pos_goal = 0;
            }
        }
        std::mem::replace(&mut self.vm, new_vm);
        Ok(())
    }

    fn step_forward(&mut self) -> Result<(), EmulError> {
        step(&mut self.vm, &self.client)?;
        if let Some(x) = self.vm.current_state() {
            self.positions.push(x.position);
        } else {
            warn!("The VM Status is {:?}, and not initialized. Pushing 0 to positions", self.vm.status());
            self.positions.push(0);
        }
        Ok(())
    }

    fn run_until(&mut self, pc: usize) -> Result<(), EmulError> {

        // If positions is 0, we haven't started the VM yet
        while *self.positions.get(self.positions.len()).unwrap_or(&0) < pc {
            step(&mut self.vm, &self.client)?;
            if let Some(x) = self.vm.current_state() {
                self.positions.push(x.position);
            }
        }
        Ok(())
    }

    /// runs vm to completion
    fn run(&mut self) -> Result<(), EmulError> {
        while !step(&mut self.vm, &self.client)? {}
        Ok(())
    }
    /* Access the VM Directly */
    fn mutate_raw<F>(&mut self, mut fun: F) -> Result<(), Error>
    where
        F: FnMut(&mut SeqTransactionVM<ByzantiumPatch>) -> Result<(), Error>
    {
        fun(&mut self.vm)
    }

    /// access the underyling vm implementation directly via the predicate F
    pub fn read_raw<F>(&self, fun: F) -> Result<(), Error>
    where
        F: Fn(&SeqTransactionVM<ByzantiumPatch>) -> Result<(), Error>
    {
        fun(&self.vm)
    }
}

/// steps the vm, querying node for any information that the VM needs
/// vm returns true when execution is finished
fn step<T>(vm: &mut SeqTransactionVM<ByzantiumPatch>, client: &Web3<T>) -> Result<bool, EmulError>
where
    T: Transport
{
    match vm.step() {
        Ok(()) => {
            Ok(true)
        },
        Err(RequireError::Account(addr)) => {
            info!("Acquiring account {:#x} for VM", addr);
            let nonce = client.eth().transaction_count(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait()?;
            let balance: U256 = client.eth().balance(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait()?; // U256
            let code: Bytes = client.eth().code(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait()?; // Bytes

            vm.commit_account(AccountCommitment::Full {
                nonce: bigint::U256(nonce.0),
                address: addr,
                balance: bigint::U256(balance.0),
                code: Rc::new(code.0),
            })?;

            Ok(false)
        },
        Err(RequireError::AccountStorage(addr, index)) => {
            info!("Acquiring account storage at {:#x}, {:#x} for VM", addr, index);
            let value = client.eth().storage(ethereum_types::H160(addr.0), ethereum_types::U256(index.0), Some(BlockNumber::Latest)).wait()?;
            vm.commit_account(AccountCommitment::Storage {
                address: addr,
                index: index,
                // unsafe needs to be used here because bigint expects 4 u64's, while web3 function gives us an array of 32 bytes
                value: bigint::M256(bigint::U256(unsafe { super::scary::non_scalar_typecast::h256_to_u256(value) } ))
            })?;
            Ok(false)
        },
        Err(RequireError::AccountCode(addr)) => {
            info!("Acquiring code at {:#x} for VM", addr);
            let code: Bytes = client.eth().code(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait()?;
            vm.commit_account(AccountCommitment::Code {
                address: addr,
                code: Rc::new(code.0)
            })?;
            Ok(false)
        },
        // the debugger is useless if execution cannot continue
        Err(err) => {
            error!("VM execution failed, unknown require! {:?}", err);
            panic!("VM Execution failed, unknown require");
        }
    }
}



#[cfg(test)]
mod test {
    use speculate::speculate;
    use bigint::{Address, Gas};
    use sputnikvm::TransactionAction;
    use super::*;
    use crate::tests::mock::MockWeb3Transport;
    use crate::tests::*;
    use std::str::FromStr;
    const simple: &'static str = include!("tests/solidity/simple.bin/SimpleStorage.bin");

    speculate! {
        describe "emulate" {
            const simple: &'static str = include!("tests/solidity/simple.bin/SimpleStorage.bin");

            before {
                pretty_env_logger::try_init();
                let mock = MockWeb3Transport::default();
                let client = web3::Web3::new(mock);
                let contract = ethabi::Contract::load(include_bytes!("tests/solidity/simple.bin/simple.json") as &[u8]).unwrap();
                let set = contract.function("set").unwrap().encode_input(&[ethabi::Token::Uint(U256::from(1337 as u64))]).unwrap();
                let get = contract.function("get").unwrap().encode_input(&[]).unwrap();
                let tx_set = ValidTransaction {
                    caller: Some(Address::from_str("94143ba98cdd5a0f3a80a6514b74c25b5bdb9b59").unwrap()), // caller
                    gas_price: Gas::one(),
                    gas_limit: Gas::max_value(),
                    // contract to call
                    action: TransactionAction::Call(bigint::H160::from_str("0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C").unwrap()),
                    value: bigint::U256::zero(),
                    input: Rc::new(set),
                    nonce: bigint::U256::zero(),
                };
                // miner
                let headers = sputnikvm::HeaderParams {
                    beneficiary: Address::from_str("11f275d2ad4390c41b150fa0efb5fb966dbc714d").unwrap(), 
                    timestamp: 1536291149 as u64,
                    number: bigint::U256::from(6285997 as u64),
                    difficulty: bigint::U256::from(3331693644712776 as u64),
                    gas_limit: bigint::Gas::from(8000000 as u64)
                };
                // make this into a macro
                let mut emul = Emulator::new(tx_set, headers, client);
                emul.mutate_raw(|vm| {
                    let code: Vec<u8> = hex::decode(simple).unwrap();
                    // commit the SimpleStorage contract to memory; 
                    // this would be like deploying a smart contract to a TestRPC
                    vm.commit_account(AccountCommitment::Full {
                        nonce: bigint::U256::zero(),
                        // contract
                        address: Address::from_str("0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C").unwrap(), 
                        balance: bigint::U256::max_value(), // never run out of gas
                        code: Rc::new(code.to_vec())
                    });

                    vm.commit_account(AccountCommitment::Full {
                        nonce: bigint::U256::zero(),
                        // miner
                        address: Address::from_str("11f275d2ad4390c41b150fa0efb5fb966dbc714d").unwrap(), 
                        balance: bigint::U256::max_value(),
                        code: Rc::new(Vec::new())
                    });

                    vm.commit_account(AccountCommitment::Full {
                        nonce: bigint::U256::zero(),
                        // caller
                        address: Address::from_str("94143ba98cdd5a0f3a80a6514b74c25b5bdb9b59").unwrap(), 
                        balance: bigint::U256::max_value(),
                        code: Rc::new(Vec::new())
                    });
                    Ok(())
                });
            }

            it "can run" {
                emul.fire(Action::Exec);
                info!("{:?}", emul.output());
            }

            it "can step forward" {
                emul.read_raw(|vm| {
                    assert_eq!(vm.current_machine().is_none(), true);
                    Ok(())
                });
                emul.fire(Action::StepForward);
                emul.fire(Action::StepForward);
                emul.read_raw(|vm| {
                    info!("current PC: {}", vm.current_state().unwrap().position);
                    assert_eq!(2, vm.current_state().unwrap().position);
                    Ok(())
                });
            }

            it "can step backward" {
                emul.fire(Action::StepForward);
                emul.fire(Action::StepForward);
                emul.fire(Action::StepForward);
                emul.fire(Action::StepBack);
                emul.read_raw(|vm| {
                    info!("Status: {:?}", vm.status());
                    info!("Positions: {:?}", emul.positions);
                    assert_eq!(2, vm.current_state().unwrap().position);
                    Ok(())
                });
            }
        }
    }
}
