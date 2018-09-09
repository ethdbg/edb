//! Emulates transaction execution and allows for real-time debugging.
//! debugs one transaction at a time (1:1 One VM, One TX)
use log::{info, error, warn, log};
use sputnikvm::{ValidTransaction, HeaderParams, SeqTransactionVM, AccountChange, errors::{RequireError, CommitError}, AccountCommitment, VM, Storage};
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
    storage: Storage,
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

/// A vm that emulates a transaction, allowing for mutations during execution
impl<T> Emulator<T> where T: Transport {
    /// Create a new Emulator
    ///
    /// example assumes the deployment of `SimpleStorage` contract to `0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C`
    ///
    /// ```rust,no_run
    ///  let web3 = web3::Web3::new(http);
    ///  let block = web3.eth().block(BlockId::Number(BlockNumber::Latest)).wait().unwrap();
    ///  let headers = sputnikvm::HeaderParams {
    ///     beneficiary: block.author, 
    ///     timestamp: block.timestamp,
    ///     number: block.number.unwrap().to_u256(),
    ///     difficulty: block.difficulty,
    ///     gas_limit: block.gas_limit
    /// };
    /// let set = contract.function("set").unwrap().encode_input(&[ethabi::Token::Uint(U256::from("1337"))]).unwrap();
    ///
    /// let tx_set = ValidTransaction {
    ///     caller: Some(Address::from_str("94143ba98cdd5a0f3a80a6514b74c25b5bdb9b59").unwrap()),
    ///     gas_price: Gas::one(),
    ///     gas_limit: Gas::from(10000000 as u64), 
    ///     action: TransactionAction::Call(bigint::H160::from_str("0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C").unwrap()),
    ///     value: bigint::U256::zero(),
    ///     input: Rc::new(set),
    ///     nonce: bigint::U256::zero(),
    /// };
    /// let emul = Emulator::new(tx_set, headers, web3);
    /// ```
    pub fn new(transaction: ValidTransaction, block: HeaderParams, client: Web3<T>) -> Self {
        Emulator {
            transaction: (transaction.clone(), block.clone()),
            vm: sputnikvm::TransactionVM::new(transaction, block),
            positions: Vec::new(),
            client,
            state_cache: Rc::new(RefCell::new(HashMap::new()))
        }
    }
    /// fire the vm, with the specified Action
    ///
    /// ```
    /// emul.fire(Action::StepForward);
    /// ```
    pub fn fire(&mut self, action: Action) -> Result<(), EmulError> {
        match action {
            Action::StepBack => self.step_back(),
            Action::StepForward => self.step_forward(),
            Action::RunUntil(pc) => self.run_until(pc),
            Action::Exec => self.run(),
            Action::Finish => self.run()
        }
    }

    /// any output that the transaction may have produced during VM execution
    pub fn output(&self) -> Vec<u8> {
        self.vm.out().into()
    }
    
    /// Chain a transaction with the state changes of the previous transaction.
    /// If header params are specified, transaction is chained with new block
    ///
    /// ```
    /// // the `get` function encoded with ETHABI crate
    /// let get = contract.function("get").unwrap().encode_input(&[]).unwrap();
    /// let tx_get = ValidTransaction {
    ///     caller: Some(Address::from_str("94143ba98cdd5a0f3a80a6514b74c25b5bdb9b59").unwrap()), // caller
    ///     gas_price: Gas::one(),
    ///     gas_limit: Gas::from(10000000 as u64),
    ///     action: TransactionAction::Call(bigint::H160::from_str("0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C").unwrap()),
    ///     value: bigint::U256::zero(),
    ///     input: Rc::new(get),
    ///     nonce: bigint::U256::zero(),
    /// };
    /// emul.chain(tx_get, None);
    /// ```
    pub fn chain(&mut self, tx: ValidTransaction, block: Option<HeaderParams>) {
        self.positions.clear();
        if let Some(new_head) = block {
            self.transaction = (tx.clone(), new_head.clone());
            self.vm = sputnikvm::TransactionVM::new(tx, new_head);
        } else {
            self.transaction.0 = tx;
            let (txinfo, block) = self.transaction.clone();
            self.vm = sputnikvm::TransactionVM::new(txinfo, block);
        }
  
    }
    /// Access the underyling vm implementation directly via the predicate F
    ///
    /// ```
    /// emul.read_raw(|vm| {
    ///     // any functions for sputnikvm::TransactionVM are now available via 'vm'
    ///     let machine = vm.current_machine();
    /// });
    /// ```
    pub fn read_raw<F>(&self, fun: F) -> Result<(), Error>
    where
        F: Fn(&SeqTransactionVM<ByzantiumPatch>) -> Result<(), Error>
    {
        fun(&self.vm)
    }

    fn step_back(&mut self) -> Result<(), EmulError> {
        info!("Positions: {:?}", self.positions);
        let mut curr_pos = 0;
        if let Some(x) = self.positions.pop() {
            curr_pos = x;
        } 
        
        let mut last_pos = 0;
        let (txinfo, header) = self.transaction.clone();
        let new_vm = sputnikvm::TransactionVM::new(txinfo, header);
        info!("Stepping vm back to last position {}, from current position {}", 
              *self.positions.get(self.positions.len() - 1).unwrap_or(&0), curr_pos);
        std::mem::replace(&mut self.vm, new_vm);
        
        // run the vm until the latest stored position
        while last_pos < *self.positions.get(self.positions.len() - 1).unwrap_or(&0) {
            self.step()?;
            if let Some(x) = self.vm.current_state() {
                last_pos = x.position;
            } else {
                panic!("Vm stepped but state is not initialized");
            }
        }
        Ok(())
    }

    fn step_forward(&mut self) -> Result<(), EmulError> {
        self.step()?;
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
            self.step()?;
            if let Some(x) = self.vm.current_state() {
                self.positions.push(x.position);
            }
        }
        Ok(())
    }

    fn run(&mut self) -> Result<(), EmulError> {
        'run: loop {
            let result = self.vm.fire();
            self.persist()?;
            if handle_requires(&result, self.state_cache.clone(), &mut self.vm, &self.client)? {
                info!("Vm  exited with code {:?}", self.vm.status());
                break 'run;
            }
        }
        Ok(())
    }
    
    /// persists any account storage that has been changed by the currently executing transaction up until the point in execution
    /// used when chaining transactions during debugging
    /// for example
    /// 
    /// ```rust,no_run
    /// let emulator = Emulator::new(tx_set, header, client);
    /// emulator.fire(Action::Exec);
    /// emulator.chain(tx_get);
    /// emulator.fire(Action::Exec);
    /// ```
    /// if debugging these two transactions seperately, it would require two different vm's (one
    /// per transaction). therefore, persistant storage
    /// 
    fn persist(&self) -> Result<(), EmulError> {
        let res = self.vm.accounts().map(|acc| {
            match acc {
                AccountChange::Full {nonce, address, balance, changing_storage, code} => {
                    info!("Changing storage: {:?}", changing_storage);
                    if changing_storage.len() > 0 && self.state_cache.borrow().contains_key(&address) {
                        for item in 0..changing_storage.len() {
                            self.state_cache
                                .borrow_mut()
                                .get_mut(&address)
                                .expect("scope conditional; qed")
                                .storage
                                .write(bigint::U256::from(item as u64), 
                                       changing_storage
                                        .read(bigint::U256::from(item as u64)).expect("Storage should not be empty; qed"))
                                .expect("require error not possible in the context of local cache; qed");
                        }
                        Ok(())
                    } else if changing_storage.len() > 0 { // if we don't have the key in our account cache yet
                        self.state_cache.borrow_mut().insert(address.clone(), Account {
                            nonce: nonce.clone(),
                            balance: balance.clone(),
                            code: code.clone(),
                            storage: changing_storage.clone().into()
                        });
                        Ok(())
                    } else { // if there is no storage to update, don't bother
                        Ok(())
                    }
                },
                AccountChange::IncreaseBalance(addr, amnt) => {
                    self.state_cache
                        .borrow_mut()
                        .get_mut(&addr)
                        .ok_or(EmulError::State(StateError::NotFound(*addr)))?
                        .balance
                        .overflowing_add(*amnt);
                    Ok(())
                },
                // Create assumes the account does not yet exist, so this will replace anything that bychance exists already locally
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
        }).collect::<Result<(), EmulError>>();
        info!("Result of persist: {:?}", res); 
        res
    }

    /// steps the vm, querying node for any information that the VM needs
    /// vm returns true when execution is finished
    fn step(&mut self) -> Result<(), EmulError> {
        let mut res = self.vm.step();
        self.persist()?;
        'require: loop {
            let req = handle_requires(&res, self.state_cache.clone(), &mut self.vm, &self.client)?;
            if req {
                break 'require;
            } else {
                res = self.vm.step();
                self.persist()?;
            }
        }
        Ok(())
    }
}


fn handle_requires<T>(
    result: &Result<(), RequireError>, 
    cache: Rc<RefCell<HashMap<bigint::H160, Account>>>,
    vm: &mut SeqTransactionVM<ByzantiumPatch>,
    client: &Web3<T>) -> Result<bool, EmulError> 
where
    T: Transport
{
    match result.clone() {
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
            info!("Local Storage: {:?}", cache);
            if cache.borrow().contains_key(&addr) && cache.borrow().get(&addr).unwrap().storage.read(index).is_ok() {
                info!("Found storage in Local Cache. Committing...");
                cache.borrow().get(&addr).and_then(|x| {
                    let res = vm.commit_account(AccountCommitment::Storage {
                        address: addr,
                        index: index,
                        value: x.storage.read(index).expect("scope is conditional; qed").clone()
                    });
                    // ignore AlreadyCommitted
                    match res {
                        Err(CommitError::InvalidCommitment) => panic!("Commitments should never be invalid; qed"),
                        _ => Some(x)
                    }
                });
                info!("Returning!");
                Ok(false)
            } else {
                let value = client.eth().storage(ethereum_types::H160(addr.0), ethereum_types::U256(index.0), Some(BlockNumber::Latest)).wait()?;
                vm.commit_account(AccountCommitment::Storage {
                    address: addr,
                    index: index,
                    // unsafe needs to be used here because bigint expects 4 u64's, while web3 function gives us an array of 32 bytes
                    value: bigint::M256(bigint::U256(unsafe { super::scary::non_scalar_typecast::h256_to_u256(value) } ))
                })?;
                Ok(false)
            }
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

    speculate! {
        describe "emulate" {
            const simple: &'static str = include!("tests/solidity/simple.bin/SimpleStorage.bin");

            before {
                pretty_env_logger::try_init();
                let mock = MockWeb3Transport::default();
                let client = web3::Web3::new(mock);
                let contract = ethabi::Contract::load(include_bytes!("tests/solidity/simple.bin/simple.json") as &[u8]).unwrap();
                let set = contract.function("set").unwrap().encode_input(&[ethabi::Token::Uint(U256::from("1337"))]).unwrap();
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
                emul.read_raw(|vm| {
                    assert_eq!(4, vm.current_state().unwrap().position);
                    Ok(())
                });
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

            it "should chain, persisting account storage" {
                emul.fire(Action::Exec).unwrap();
                emul.chain(tx_get, None);
                emul.fire(Action::Exec).unwrap();
                let out = emul.output();
                assert_eq!(U256::from("1337"), U256::from(out.as_slice()));
            }
        }
    }
}
