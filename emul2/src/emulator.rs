//! Emulates transaction execution and allows for real-time debugging
//! Debugs one transaction at a time (1:1 One VM, One TX)
use log::{info, error, warn, log};
use serde_derive::*;
use sputnikvm::{ValidTransaction, HeaderParams, SeqTransactionVM, VMStatus, AccountChange, errors::{RequireError, CommitError}, AccountCommitment, VM};
use futures::future::Future;
use sputnikvm_network_foundation::ByzantiumPatch;
use failure::Error;
use web3::{
    api::Web3,
    Transport,
    types::{BlockNumber, U256, Bytes},
};
use std::{
    rc::Rc,
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
};
use super::{
    err::{EmulError, StateError},
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
#[derive(Debug, Clone)]
struct Account {
    nonce: bigint::U256,
    balance: bigint::U256,
    storage: HashMap<bigint::U256, bigint::M256>,
    code: Rc<Vec<u8>>
}

/// Emulation Object
pub struct Emulator<T: Transport> {
    vm: SeqTransactionVM<ByzantiumPatch>,
    positions: Vec<usize>,
    transaction: (ValidTransaction, HeaderParams),
    client: web3::Web3<T>,
    state_cache: Rc<RefCell<HashMap<bigint::H160, Account>>>
}

/// a vm that emulates a transaction, allowing for mutations during execution
impl<T> Emulator<T> where T: Transport {
    /// Create a new Emulator
    pub fn new(transaction: ValidTransaction, header: HeaderParams, client: Web3<T>) -> Self {
        Emulator {
            transaction: (transaction.clone(), header.clone()),
            vm: sputnikvm::TransactionVM::new(transaction, header),
            positions: Vec::new(),
            client,
            state_cache: Rc::new(RefCell::new(HashMap::new()))
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
                AccountChange::Full{nonce, address, balance, code, changing_storage} => {
                    info!("Committing account {:#x} from previous vm", address);
                    new_vm.commit_account(AccountCommitment::Full {
                        nonce: nonce.clone(), 
                        address: address.clone(), 
                        balance: balance.clone(), 
                        code: code.clone(),
                    })?;
                    if changing_storage.len() > 0 {
                        info!("Committing storage for account {:#x} from previous vm", address);
                        for i in 0..changing_storage.len() {
                            new_vm.commit_account(AccountCommitment::Storage {
                                address: address.clone(),
                                index: bigint::U256::from(i),
                                value: changing_storage.read(bigint::U256::from(i))
                                    .expect("Storage should exist in changing storg is not 0"),
                            });
                        }
                    }
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
        let mut curr_pos = 0;
        if let Some(x) = self.positions.pop() {
            curr_pos = x;
        } else { // if nothing is on the positions stack, return a new VM with any accounts that may have been commmitted
            let new_vm = self.new_vm()?;
            std::mem::replace(&mut self.vm, new_vm);
            return Ok(());
        }
        
        let mut last_pos = 0;
        let mut new_vm = self.new_vm()?; 
        info!("Stepping vm back to last position {}, from current position {}", last_pos, curr_pos);
        // run the vm until the latest stored position
        while last_pos < *self.positions.get(self.positions.len() - 1).unwrap_or(&0) {
            step(&mut new_vm, self.state_cache.clone(), &self.client)?;
            if let Some(x) = new_vm.current_state() {
                last_pos = x.position;
            } else {
                panic!("Vm stepped but state is not initialized");
            }
        }
        std::mem::replace(&mut self.vm, new_vm);
        Ok(())
    }

    fn step_forward(&mut self) -> Result<(), EmulError> {
        step(&mut self.vm, self.state_cache.clone(), &self.client)?;
        if let Some(x) = self.vm.current_state() {
            self.positions.push(x.position);
        } else {
            warn!("The VM Status is {:?} but not initialized. Pushing 0 to positions", self.vm.status());
            self.positions.push(0);
        }
        Ok(())
    }

    fn run_until(&mut self, pc: usize) -> Result<(), EmulError> {

        // If position is 0, we haven't started the VM yet
        while *self.positions.get(self.positions.len()).unwrap_or(&0) < pc {
            step(&mut self.vm, self.state_cache.clone(), &self.client)?;
            if let Some(x) = self.vm.current_state() {
                self.positions.push(x.position);
            }
        }
        Ok(())
    }

    /// runs vm to completion
    fn run(&mut self) -> Result<(), EmulError> {
        'run: loop {
            let result = self.vm.fire();
            if handle_requires(result, self.state_cache.clone(), &mut self.vm, &self.client)? {
                info!("Vm  exited with code {:?}", self.vm.status());
                break 'run;
            }
        }
        Ok(())
    }

    /// access the underyling vm implementation directly via the predicate F
    pub fn read_raw<F>(&self, fun: F) -> Result<(), Error>
    where
        F: Fn(&SeqTransactionVM<ByzantiumPatch>) -> Result<(), Error>
    {
        fun(&self.vm)
    }
    
    /// persists any account storage that has been changed by the currently executing transaction up until the point in execution
    /// used when chaining transactions during debugging
    /// for example
    /// 
    /// ```C++
    /// let emulator = Emulator::new(tx_set, header, client);
    /// emulator.fire(Action::Exec);
    /// emulator.chain(tx_get);
    /// emulator.fire(Action::Exec);
    /// ```
    /// if debugging these two transactions seperately, it would require two different vm's (one
    /// per transaction). therefore, persistant storage
    /// 
    pub fn persist(&self) -> Result<(), Error> {
        self.vm.accounts().map(|acc| {
            match acc {
                AccountChange::Full {nonce, address, balance, changing_storage, code} => {
                    self.state_cache.borrow_mut().insert(address.clone(), Account {
                        nonce: nonce.clone(),
                        balance: balance.clone(),
                        code: code.clone(),
                        storage: changing_storage.clone().into()
                    });
                    Ok(())
                },
                AccountChange::IncreaseBalance(addr, amnt) => {
                    let acc = self.state_cache
                        .borrow_mut()
                        .get_mut(&addr)
                        .ok_or(EmulError::State(StateError::NotFound(*addr)))?
                        .balance
                        .overflowing_add(*amnt);
                    Ok(())
                },
                AccountChange::Create {nonce, address, balance, storage, code} => {
                    self.state_cache.borrow_mut().insert(address.clone(), Account {
                        nonce: nonce.clone(),
                        balance: balance.clone(),
                        code: code.clone(),
                        storage: storage.clone().into()

                    });
                    Ok(())
                },
                AccountChange::Nonexist(addr) => {
                    if self.state_cache.borrow().contains_key(&addr) {
                        self.state_cache.borrow_mut().remove(&addr);
                    }
                    Ok(())
                }
            }
        }).collect::<Result<(), EmulError>>()?;
        Ok(())
    }
}

/// steps the vm, querying node for any information that the VM needs
/// vm returns true when execution is finished
fn step<T>(vm: &mut SeqTransactionVM<ByzantiumPatch>, cache: Rc<RefCell<HashMap<bigint::H160, Account>>>, client: &Web3<T>) -> Result<bool, EmulError>
where
    T: Transport
{
    let result = vm.step();
    handle_requires(result, cache, vm, client)
}

fn handle_requires<T>(
    result: Result<(), RequireError>, 
    cache: Rc<RefCell<HashMap<bigint::H160, Account>>>,
    vm: &mut SeqTransactionVM<ByzantiumPatch>,
    client: &Web3<T>) -> Result<bool, EmulError> 
where
    T: Transport
{

    match result {
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
            step(vm, cache, client)
        },
        Err(RequireError::AccountStorage(addr, index)) => {
            info!("Acquiring account storage at {:#x}, {:#x} for VM", addr, index);
            if cache.borrow().contains_key(&addr) && cache.borrow().get(&addr).unwrap().storage.contains_key(&index) {
                info!("Found storage in Local Cache. Committing...");
                cache.borrow().get(&addr).and_then(|x| {
                    vm.commit_account(AccountCommitment::Storage {
                        address: addr,
                        index: index,
                        value: x.storage.get(&index).expect("scope is conditional; qed").clone()
                    });
                    Some(x)
                });
                return Ok(false); // early return if we found the account and value in cache
            }
            let value = client.eth().storage(ethereum_types::H160(addr.0), ethereum_types::U256(index.0), Some(BlockNumber::Latest)).wait()?;
            vm.commit_account(AccountCommitment::Storage {
                address: addr,
                index: index,
                // unsafe needs to be used here because bigint expects 4 u64's, while web3 function gives us an array of 32 bytes
                value: bigint::M256(bigint::U256(unsafe { super::scary::non_scalar_typecast::h256_to_u256(value) } ))
            })?;
            step(vm, cache, client)
        },
        Err(RequireError::AccountCode(addr)) => {
            info!("Acquiring code at {:#x} for VM", addr);
            let code: Bytes = client.eth().code(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait()?;
            vm.commit_account(AccountCommitment::Code {
                address: addr,
                code: Rc::new(code.0)
            })?;
            step(vm, cache, client)
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
                    gas_limit: Gas::from(10000000 as u64),
                    // contract to call
                    action: TransactionAction::Call(bigint::H160::from_str("0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C").unwrap()),
                    value: bigint::U256::zero(),
                    input: Rc::new(set),
                    nonce: bigint::U256::zero(),
                };
                let tx_get = ValidTransaction {
                    caller: Some(Address::from_str("94143ba98cdd5a0f3a80a6514b74c25b5bdb9b59").unwrap()), // caller
                    gas_price: Gas::one(),
                    gas_limit: Gas::from(10000000 as u64),
                    // contract to call
                    action: TransactionAction::Call(bigint::H160::from_str("0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C").unwrap()),
                    value: bigint::U256::zero(),
                    input: Rc::new(get),
                    nonce: bigint::U256::zero(),
                };
                // miner
                let headers = sputnikvm::HeaderParams {
                    beneficiary: Address::from_str("11f275d2ad4390c41b150fa0efb5fb966dbc714d").unwrap(), 
                    timestamp: 1536291149 as u64,
                    number: bigint::U256::from(6285997 as u64),
                    difficulty: bigint::U256::from(3331693644712776 as u64),
                    gas_limit: bigint::Gas::from(80000000 as u64)
                };
                // make this into a macro
                let mut emul = Emulator::new(tx_set, headers, client);
                let code: Vec<u8> = hex::decode(simple).unwrap();
            }

            it "can run" {
                emul.fire(Action::Exec).unwrap();
            }

            it "can step forward" {
                emul.read_raw(|vm| {
                    assert_eq!(vm.current_machine().is_none(), true);
                    Ok(())
                }).unwrap();
                emul.fire(Action::StepForward).unwrap();
                emul.fire(Action::StepForward).unwrap();
                emul.read_raw(|vm| {
                    info!("current PC: {}", vm.current_state().unwrap().position);
                    assert_eq!(2, vm.current_state().unwrap().position);
                    Ok(())
                }).unwrap();
            }

            it "can step backward" {
                emul.fire(Action::StepForward).unwrap();
                emul.fire(Action::StepForward).unwrap();
                emul.fire(Action::StepForward).unwrap();
                emul.fire(Action::StepBack).unwrap();
                emul.read_raw(|vm| {
                    assert_eq!(2, vm.current_state().unwrap().position);
                    Ok(())
                }).unwrap();
            }

            it "can execute the entire program" {
                emul.fire(Action::Exec).unwrap();
                emul.read_raw(|vm| {
                    info!("VM PC: {}", vm.current_state().unwrap().position);
                    Ok(())
                }).unwrap();
            }

            it "can step and then finish the execution" {
                emul.fire(Action::StepForward).unwrap();
                emul.fire(Action::StepForward).unwrap();
                emul.fire(Action::StepBack).unwrap();
                emul.fire(Action::Finish).unwrap();
            }

            it "can set and get" {
                emul.fire(Action::Exec).unwrap();
                let (tx, header) = emul.transaction.clone();
                emul.persist();
                info!("Storage: {:?}", emul.state_cache);
                sputnikvm::TransactionVM::with_previous(tx_get, header, &emul.vm);
                emul.fire(Action::Exec).unwrap();
                let out = emul.output();
                info!("Output: {:?}", out);
                info!("Output as hex: {}", hex::encode(&out.as_slice()));
            }
        }
    }
}
