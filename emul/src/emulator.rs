//! Emulates transaction execution and allows for real-time debugging.
//! debugs one transaction at a time (1:1 One VM, One TX)
use sputnikvm::{Opcode, VMStatus, ValidTransaction, TransactionAction, HeaderParams, SeqTransactionVM, SeqMemory, AccountChange, errors::{RequireError, CommitError}, AccountCommitment, VM, Storage, PC};
use sputnikvm_network_foundation::ByzantiumPatch;
use web3::{ api::Web3, Transport, types::{BlockNumber, U256, Bytes}};
use futures::future::Future;
use failure::Error;
use log::*;
use std::{ rc::Rc, cell::RefCell, collections::{HashMap} };
use super::err::{EmulError, StateError};

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

impl Account {
    // Merge a Storage struct with local cache
    fn merge(&mut self, storage: Storage) -> () {
        let map: HashMap<bigint::U256, bigint::M256> = storage.into();
        self.storage.extend(map.iter())
    }
}

/// Emulation Object
pub struct Emulator<T: Transport> {
    vm: SeqTransactionVM<ByzantiumPatch>,
    positions: Vec<usize>,
    transaction: (ValidTransaction, HeaderParams),
    client: web3::Web3<T>,
    state_cache: Rc<RefCell<HashMap<bigint::H160, Account>>>,
    // the amount of instructions have we stepped
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
            state_cache: Rc::new(RefCell::new(HashMap::new())),
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

    pub fn memory(&self) -> Result<&SeqMemory<ByzantiumPatch>, EmulError> {
        Ok(&self.vm.current_state().ok_or(EmulError::CouldNotAcquireVm)?.memory)
    }

    pub fn storage(&self) -> Option<HashMap<bigint::U256, bigint::M256>> {
        self.state_cache.borrow()
            .get(&self.resident_address())
            .map(|acc| acc.storage.clone())
    }

    // The address of the contract that is being debugged
    fn resident_address(&self) -> bigint::H160 {
        let (tx, _) = &self.transaction;
        match tx.action {
            TransactionAction::Call(addr) => addr,
            _ => panic!("Cannot debug creation of contracts")
        }
    }
    fn run_until(&mut self, opcode_pos: usize) -> Result<(), EmulError> {
        // If position is 0, we haven't started the VM yet
        while *self.positions.last().unwrap_or(&0) < opcode_pos {
            self.step_forward()?;
        }
        Ok(())
    }

    pub fn finished(&self) -> bool {
        match self.vm.status() {
            VMStatus::Running => false,
            VMStatus::ExitedOk => true,
            VMStatus::ExitedErr(err) => {
                warn!("VM Exited with err {:?}", err);
                true
            },
            VMStatus::ExitedNotSupported(err) => {
                warn!("VM Exited due to unsupported operation {:?}", err);
                true
            }
        }
    }

    /// any output that the transaction may have produced during VM execution
    pub fn output(&self) -> Vec<u8> {
        self.vm.out().into()
    }

    /// Get the Current Runtime PC
    pub fn pc(&self) -> Option<PC<ByzantiumPatch>> {
        if let Some(mach) = self.vm.current_machine() {
            Some(mach.pc())
        } else {
            None
        }
    }

    /// get bytecode position
    pub fn offset(&self) -> Result<usize, EmulError> {
        //Ok(self.vm.current_machine().ok_or(EmulError::CouldNotAcquireVm)?.pc().opcode_position())
        Ok(self.vm.current_machine().ok_or(EmulError::CouldNotAcquireVm)?.pc().position())
    }

    /// return the instruction position from an opcode offset
    pub fn instruction(&self) -> Result<usize, EmulError> {
        Ok(Self::into_instruction(self.offset()?, self.vm.current_machine().ok_or(EmulError::CouldNotAcquireVm)?.pc().code()))
    }

    fn into_instruction(position: usize, code: &[u8]) -> usize {
        let mut opcode_pos = 0;
        let mut instruction_pos = 0;
        'interpreter: loop {
            let instruction = code[opcode_pos];
            // check for CBOR-encoded AUXDATA first. this will always start with 0xa1
            // this should be unique enough to avoid conflicts with other languages
            // So far, AFAIK, Solidity is the only language that adds an extra 'metadata' portion
            // to the end of runtime bytecode
            match instruction {
                0xa1 => {
                    let code_slice = &code[opcode_pos..];
                                                  // b     z     z      r     0
                    if &code_slice[1..9] == &[0x65, 0x62, 0x7a, 0x7a, 0x72, 0x30, 0x58, 0x20] {
                        break 'interpreter;
                    } else {
                        continue;
                    }
                }
                _ => (),
            };
            match Opcode::from(instruction) {
                Opcode::PUSH(bytes) => {
                    opcode_pos += bytes + 1;
                },
                _ => {
                    opcode_pos += 1;
                },
            }
            if opcode_pos >= position {
                break 'interpreter;
            }
            instruction_pos += 1;
        }
        debug!("Instruction Position {}", instruction_pos);
        instruction_pos
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
    pub fn read_raw<F>(&self, mut fun: F) -> Result<(), Error>
    where
        F: FnMut(&SeqTransactionVM<ByzantiumPatch>) -> Result<(), Error>
    {
        fun(&self.vm)
    }

    fn step_back(&mut self) -> Result<(), EmulError> {
        let mut last_pos = 0;
        let (txinfo, header) = self.transaction.clone();
        let new_vm = sputnikvm::TransactionVM::new(txinfo, header);
        std::mem::replace(&mut self.vm, new_vm);

        // run the vm until the latest stored position
        while last_pos < *self.positions.last().unwrap_or(&0) {
            self.step()?;
            if let Some(x) = self.vm.current_machine() {
                last_pos = x.pc().opcode_position();
            } else {
                panic!("Vm stepped but state is not initialized");
            }
        }
        Ok(())
    }

    fn step_forward(&mut self) -> Result<(), EmulError> {
        self.step()?;
        if let Some(x) = self.vm.current_machine() {
            self.positions.push(x.pc().opcode_position());
        } else {
            self.positions.push(0);
        }

        Ok(())
    }

    fn run(&mut self) -> Result<(), EmulError> {
        'run: loop {
            let result = self.vm.fire();
            self.persist()?;
            if handle_requires(&result, self.state_cache.clone(), &mut self.vm, &self.client)? {
                break 'run;
            }
        }
        Ok(())
    }

    /// steps the vm, querying node for any information that the VM needs
    /// vm returns true when execution is finished
    fn step(&mut self) -> Result<(), EmulError> {
        let mut res = self.vm.step();
        trace!("VM Step {:?}", res);
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
                    if changing_storage.len() > 0 && self.state_cache.borrow().contains_key(&address) {
                        self.state_cache.borrow_mut()
                            .get_mut(&address)
                            .expect("Scope is conditional; qed")
                            .merge(changing_storage.clone());
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
        res
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
            info!("Acquiring balance, code, and nonce of account {:#x} for VM", addr);
            let nonce = client.eth().transaction_count(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait()?;
            debug!("Nonce: {:#x}", nonce);
            let balance: U256 = client.eth().balance(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait()?; // U256
            debug!("Balance: {:#x}", balance);
            let mut code = client.eth().code(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait(); // Bytes
            debug!("Code: {:x?}", code);
            if code.is_err() {
                code = Ok(Bytes(vec![0]));
            }
            let code = code.unwrap();
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
            debug!("Local Storage: {:?}", cache);
            if cache.borrow().contains_key(&addr) && cache.borrow().get(&addr).unwrap().storage.get(&index).is_some() {
                trace!("Found storage in Local Cache. Committing...");
                cache.borrow().get(&addr).and_then(|x| {
                    let res = vm.commit_account(AccountCommitment::Storage {
                        address: addr,
                        index: index,
                        value: x.storage.get(&index).expect("scope is conditional; qed").clone()
                    });
                    // ignore AlreadyCommitted
                    match res {
                        Err(CommitError::InvalidCommitment) => panic!("Commitments should never be invalid; qed"),
                        _ => Some(x)
                    }
                });
                Ok(false)
            } else {
                let value = client.eth().storage(ethereum_types::H160(addr.0), ethereum_types::U256(index.0), Some(BlockNumber::Latest)).wait()?;
                debug!("Committing account {:#x} with storage at {:#x} that is {:#x} to VM", addr, index, value);
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
    use edb_test_helpers as edbtest;

    speculate! {
        describe "emulate" {

            before {
                pretty_env_logger::try_init();
                let mock = edbtest::MockWeb3Transport::default();
                let client = web3::Web3::new(mock);
                let contract = edbtest::abi(edbtest::SIMPLE_STORAGE_ABI);
                let set = contract.function("set").unwrap().encode_input(&[ethabi::Token::Uint(U256::from("1337"))]).unwrap();
                let get = contract.function("get").unwrap().encode_input(&[]).unwrap();
                let tx_set = ValidTransaction {
                    caller: Some(edbtest::bigint_addr(edbtest::ADDR_CALLER)), // caller
                    gas_price: Gas::one(),
                    gas_limit: Gas::from(10000000 as u64),
                    // contract to call
                    action: TransactionAction::Call(edbtest::bigint_addr(edbtest::SIMPLE_STORAGE_ADDR)),
                    value: bigint::U256::zero(),
                    input: Rc::new(set),
                    nonce: bigint::U256::zero(),
                };
                let tx_get = ValidTransaction {
                    caller: Some(edbtest::bigint_addr(edbtest::ADDR_CALLER)), // caller
                    gas_price: Gas::one(),
                    gas_limit: Gas::from(10000000 as u64),
                    // contract to call
                    action: TransactionAction::Call(edbtest::bigint_addr(edbtest::SIMPLE_STORAGE_ADDR)),
                    value: bigint::U256::zero(),
                    input: Rc::new(get),
                    nonce: bigint::U256::zero(),
                };
                // miner
                let headers = sputnikvm::HeaderParams {
                    beneficiary: edbtest::bigint_addr(edbtest::MINER),
                    timestamp: 1536291149 as u64,
                    number: bigint::U256::from(6285997 as u64),
                    difficulty: bigint::U256::from(3331693644712776 as u64),
                    gas_limit: bigint::Gas::from(80000000 as u64)
                };
                // make this into a macro
                let mut emul = Emulator::new(tx_set, headers, client);
                // let code: Vec<u8> = hex::decode(simple).unwrap();
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
                    trace!("current PC: {}", vm.current_state().unwrap().position);
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
                    trace!("Current PC: {}", vm.current_machine().unwrap().pc().position());
                    trace!("Current Opcode Pos: {}", vm.current_machine().unwrap().pc().opcode_position());
                    trace!("Code: {:?}", vm.current_machine().unwrap().pc().code());
                    trace!("Next Opcode: {:?}", vm.current_machine().unwrap().pc().peek_opcode().unwrap());
                    trace!("Next Instruction: {:?}", vm.current_machine().unwrap().pc().peek().unwrap());
                    Ok(())
                }).unwrap();
                emul.fire(Action::StepBack).unwrap();
                emul.read_raw(|vm| {
                    assert_eq!(2, vm.current_state().unwrap().position);
                    Ok(())
                }).unwrap();
            }

            it "can execute the entire program" {
                emul.fire(Action::Exec).unwrap();
                emul.read_raw(|vm| {
                    trace!("VM PC: {}", vm.current_state().unwrap().position);
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

            it "should get an instruction offset from opcode offset" {
                // fn into_instruction(position: usize, code: &[u8]) -> usize {
                // PUSH1 0x80 PUSH3 0x20 0x31 0x32 ADD DIV PUSH6 0x10 0x40 0x10 0x10 0x70 0x23
                let code: [u8; 15] = [0x60, 0x80, 0x62, 0x20, 0x31, 0x32, 0x01, 0x04, 0x65, 0x10, 0x40, 0x10, 0x10, 0x70, 0x23];
                let offset = Emulator::<edbtest::MockWeb3Transport>::into_instruction(7, &code);
                assert_eq!(offset, 3)
            }
        }
    }
}
