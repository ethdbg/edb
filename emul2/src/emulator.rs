//! Emulates transaction execution and allows for real-time debugging
use log::{info, error, log};
use sputnikvm::{ValidTransaction, HeaderParams, SeqTransactionVM, errors::{RequireError, CommitError}, AccountCommitment, VM};
use futures::future::Future;
use sputnikvm_network_foundation::ByzantiumPatch;
use failure::Error;
use web3::{
    api::Web3,
    Transport,
    types::{BlockNumber, U256, Bytes},
};
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

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

    pub fn fire(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::StepBack => self.step_back(),
            Action::StepForward => self.step_forward(),
            Action::RunUntil(pc) => self.run_until(pc),
            Action::Exec => self.run(),
            Action::Finish => self.run()
        }
    }

    fn output(&self) -> Vec<u8> {
        self.vm.out().into()
    }

    fn step_back(&mut self) -> Result<(), Error> {
        let mut last_pos = 0;
        if let Some(x) = self.positions.pop() {
            last_pos = x;
        }

        let mut pos_goal = 0;
        let (txinfo, header) = self.transaction.clone();
        let mut new_vm = sputnikvm::TransactionVM::new(txinfo, header);
        while pos_goal < last_pos {
            step(&mut new_vm, &self.client)?;
            let state = new_vm.current_state().unwrap();
            pos_goal = state.position;
        }
        std::mem::replace(&mut self.vm, new_vm);
        Ok(())
    }

    fn step_forward(&mut self) -> Result<(), Error> {
        step(&mut self.vm, &self.client)?;
        let state = self.vm.current_state().unwrap();
        self.positions.push(state.position);
        Ok(())
    }

    fn run_until(&mut self, pc: usize) -> Result<(), Error> {

        // If positions is 0, we haven't started the VM yet
        while *self.positions.get(self.positions.len()).unwrap_or(&0) < pc {
            step(&mut self.vm, &self.client)?;
            let state = self.vm.current_state().unwrap();
            self.positions.push(state.position.clone());
        }
        Ok(())
    }

    /// runs vm to completion
    fn run(&mut self) -> Result<(), Error> {
        while !step(&mut self.vm, &self.client)? {}
        Ok(())
    }

    // access the vm directly
    fn mutate_raw<F>(&mut self, mut fun: F) -> Result<(), Error>
    where
        F: FnMut(&mut SeqTransactionVM<ByzantiumPatch>) -> Result<(), Error>
    {
        fun(&mut self.vm)
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

            let commit_res = vm.commit_account(AccountCommitment::Full {
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
    use std::str::FromStr;
    const simple: &'static str = include!("tests/solidity/simple.bin/SimpleStorage.bin");
/*
    #[test]
    fn step() {
        let mock = MockWeb3Transport::default();
        let client = web3::Web3::new(mock);
        let contract = ethabi::Contract::load(include_bytes!("tests/solidity/simple.bin/simple.json") as &[u8]).unwrap();
        let tx = ValidTransaction {
            caller: Some(Address::random()),
            gas_price: Gas::one(),
            gas_limit: Gas::max_value(),
            action: TransactionAction::Call(H160::from_str("0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C")),
            value: U256::zero(),
            input: ,
            nonce: U256::zero(),
        };
        /// make this into a macro
        let emul = Emulator::new();
        emul.mutate_raw(|vm| {
            let code: Vec<u8> = simple.parse();
            // commit the SimpleStorage contract to memory; this would be like deploying a smart contract to a TestRPC
            vm.commit_account(AccountCommitment::Full {
                nonce: 0,
                address: Address::from_str("0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C").unwrap(),
                balance: U256::max_value(), // never run out of gas
                code: Rc::new(code)
            });
        });
    }
*/
    // pub fn new(transaction: ValidTransaction, header: HeaderParams, client: Web3<T>) -> Self {
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
                    action: TransactionAction::Call(bigint::H160::from_str("0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C").unwrap()), // contract to call
                    value: bigint::U256::zero(),
                    input: Rc::new(set),
                    nonce: bigint::U256::zero(),
                };
                let headers = sputnikvm::HeaderParams {
                    beneficiary: Address::from_str("11f275d2ad4390c41b150fa0efb5fb966dbc714d").unwrap(), // miner
                    timestamp: 1536291149 as u64,
                    number: bigint::U256::from(6285997 as u64),
                    difficulty: bigint::U256::from(3331693644712776 as u64),
                    gas_limit: bigint::Gas::from(8000000 as u64)
                };
                // make this into a macro
                let mut emul = Emulator::new(tx_set, headers, client);
                emul.mutate_raw(|vm| {
                    let code: Vec<u8> = hex::decode(simple).unwrap();
                    // commit the SimpleStorage contract to memory; this would be like deploying a smart contract to a TestRPC
                    vm.commit_account(AccountCommitment::Full {
                        nonce: bigint::U256::zero(),
                        address: Address::from_str("0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C").unwrap(), // contract
                        balance: bigint::U256::max_value(), // never run out of gas
                        code: Rc::new(code.to_vec())
                    });

                    vm.commit_account(AccountCommitment::Full {
                        nonce: bigint::U256::zero(),
                        address: Address::from_str("11f275d2ad4390c41b150fa0efb5fb966dbc714d").unwrap(), // miner
                        balance: bigint::U256::max_value(),
                        code: Rc::new(Vec::new())
                    });

                    vm.commit_account(AccountCommitment::Full {
                        nonce: bigint::U256::zero(),
                        address: Address::from_str("94143ba98cdd5a0f3a80a6514b74c25b5bdb9b59").unwrap(), // caller
                        balance: bigint::U256::max_value(),
                        code: Rc::new(Vec::new())
                    });
                    info!("Accounts: {:?}", vm.accounts());
                    Ok(())
                });
            }

            it "can run" {
                emul.fire(Action::Exec);
                info!("{:?}", emul.output());
            }

        }
    }
}
