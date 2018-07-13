use ethcore::executive::{Executive, TransactOptions, contract_address};
use ethcore::executed::{Executed, ExecutionResult};
use ethcore::state::{Backend as StateBackend, State, Substate, CleanupMode};
use ethcore::machine::EthereumMachine as Machine;
use ethcore::trace::{self, Tracer, VMTracer, FlatTrace, VMTrace};
use ethcore::error::ExecutionError;
use ethereum_types::{U256, U512};
use bytes::{Bytes, BytesRef};
use transaction::{Action, SignedTransaction};
use vm::{self, Schedule, ActionParams, ActionValue, EnvInfo, CleanDustMode};
use evm::{FinalizationResult, Finalize, CallType, CostType};
use evm::stack::{VecStack};
use ethcore_io::LOCAL_STACK_SIZE;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use externalities::*;

// TODO: replace static strings with actual errors
// a composition struct of Executive
// can't be true extension trait because Executive contains private fields
// that cannot be accessed.
// luckily these fields are passed in by reference, which means we can create a 
// new struct without losing much performance from passing objects around
// however, state is a mutable reference. A `StateDiff` is returned wherever `State` 
// is used, but I will guess allocating space for a small struct via ::new()
// is preferable to iterating through a variably-sized BTree everytime a state change
// may occur

#[derive(Debug,  PartialEq, Clone)]
pub struct DebugExecuted<T = FlatTrace, V = VMTrace> {
    pub executed: Option<Executed<T,V>>,
    is_complete: bool,
    pub mem: Vec<u8>,
    pub pc: usize,
    pub stack: VecStack<U256>,
}


pub trait ExecutiveExt<'a, B: StateBackend> {
   
    /// like transact_with_tracer + transact_virtual but with real-time debugging 
    /// functionality. Execute a transaction within the debug context
    /// pc = Program Counter (where to stop execution)
    fn begin_debug_transact<T, V>(&'a mut self, 
                           t: &SignedTransaction, 
                           check_nonce: bool,
                           output_from_create: bool,
                           mut tracer: T,
                           mut vm_tracer: V,
                           pc: usize
    ) -> Result<DebugExecuted<T::Output, V::Output>, ExecutionError> where T: Tracer, V: VMTracer;
                                         // returns a result with InstructionSnapshot
                                         // if no breakpoints set, returns error
    // returns result with Output (Completed Tx) or InstructionSnapshot (still needs resume)
    fn resume_debug_transact<T,V>(&'a mut self, 
                             t: &SignedTransaction, 
                             options: TransactOptions<T, V>, 
                             pc: usize
        ) -> Result<DebugExecuted<T::Output, V::Output>, ExecutionError> where T: Tracer, V: VMTracer; 
    
    /// call a contract function with contract params
    /// until 'PC' (program counter) is hit
    fn debug_call<T,V>(&'a mut self,
                           pc: usize,
                           params: ActionParams,
                           substate: &mut Substate,
                           mut output: BytesRef,
                           tracer: &mut T,
                           vm_tracer: &mut V
    ) -> vm::Result<FinalizationResult> where T: Tracer, V: VMTracer;
    
    /// Execute VM until it hits the program counter
    fn exec_step_vm<T, V>(&'a mut self, 
                          pc: usize, 
                          schedule: Schedule, 
                          params: ActionParams, 
                          unconfirmed_substate: &mut Substate, 
                          output_policy: OutputPolicy, 
                          tracer: &'a mut T, 
                          vm_tracer: &'a mut V);
       /* -> vm::Result<FinalizationResult> where T: Tracer, V:VMTracer */
 /*   {   
        // LOCAL_STACK_SIZE is a `Cell`
        let local_stack_size = LOCAL_STACK_SIZE.with(|sz| sz.get());
        println!("STACK SIZE {}", local_stack_size);
        
    }*/
}


// TODO: add enum type that allows a config option to choose whether to execute with or without 
// validation
impl<'a, B: 'a + StateBackend> ExecutiveExt<'a, B> for Executive<'a, B> {

    /// like transact_with_tracer + transact_virtual but with real-time debugging 
    /// functionality. Execute a transaction within the debug context
    fn begin_debug_transact<T,V>(&'a mut self, 
                                 t: &SignedTransaction,
                                 check_nonce: bool,
                                 output_from_create: bool,
                                 mut tracer: T,
                                 mut vm_tracer: V,
                                 pc: usize) 
        -> Result<DebugExecuted<T::Output, V::Output>, ExecutionError>
            where T: Tracer, V: VMTracer
    {
        /* setup a virtual transaction */
        let sender = t.sender();
        let balance = self.state.balance(&sender)?;
        let needed_balance = t.value.saturating_add(t.gas.saturating_mul(t.gas_price));
        if balance < needed_balance {
            self.state.add_balance(&sender, &(needed_balance - balance), CleanupMode::NoEmpty)?;
        }
        /* virt tx setup */ // might not need the above block

        let nonce = self.state.nonce(&sender)?;
        let schedule = self.machine.schedule(self.info.number);
        let base_gas_required = U256::from(t.gas_required(&schedule));

        // Might not want all this validation
        if t.gas < base_gas_required {
            return Err(ExecutionError::NotEnoughBaseGas { required: base_gas_required, got: t.gas }); 
        }

        if !t.is_unsigned() && check_nonce && schedule.kill_dust != CleanDustMode::Off && 
            !self.state.exists(&sender)? {
                return Err(ExecutionError::SenderMustExist);
        }
 
        let init_gas = t.gas - base_gas_required;
        
        // tx nonce validation
        if check_nonce && t.nonce != nonce {
            return Err(ExecutionError::InvalidNonce { expected: nonce, got: t.nonce });
        }

        // validate if tx fits into block
        if self.info.gas_used + t.gas > self.info.gas_limit {
            return Err(ExecutionError::BlockGasLimitReached {
                gas_limit: self.info.gas_limit,
                gas_used: self.info.gas_used,
                gas: t.gas
            });
        }
        
        let balance = self.state.balance(&sender)?;
        let gas_cost = t.gas.full_mul(t.gas_price);
        let total_cost = U512::from(t.value) + gas_cost;
        
        // validate if user can afford tx
        let balance512 = U512::from(balance);
        if balance512 < total_cost {
            return Err(ExecutionError::NotEnoughCash { required: total_cost, got: balance512 });
        }

        let mut substate = Substate::new();

        if !schedule.eip86 || !t.is_unsigned() {
            self.state.inc_nonce(&sender)?;
        }
    
        self.state.sub_balance(&sender, 
                               &U256::from(gas_cost), 
                               &mut substate.to_cleanup_mode(&schedule))?;

        let(result, output) = match t.action {
            Action::Create => { // no debugging for create actions yet
                let (new_address, code_hash) = 
                    contract_address(self.machine.create_address_scheme(self.info.number), 
                        &sender, &nonce, &t.data);
                let params = ActionParams {
                    code_address: new_address.clone(),
                    code_hash,
                    address: new_address,
                    sender: sender.clone(),
                    origin: sender.clone(),
                    gas: init_gas,
                    gas_price: t.gas_price,
                    value: ActionValue::Transfer(t.value),
                    code: Some(Arc::new(t.data.clone())),
                    data: None,
                    call_type: CallType::None,
                    params_type: vm::ParamsType::Embedded,
                };
                let mut out = if output_from_create { Some(vec![])} else { None };
                (self.create(params, &mut substate, &mut out, &mut tracer, &mut vm_tracer), out.unwrap_or_else(Vec::new))
            },
            Action::Call(ref address) => {
                let params = ActionParams {
                    code_address: address.clone(),
                    address: address.clone(),
                    sender: sender.clone(),
                    origin: sender.clone(),
                    gas: init_gas,
                    gas_price: t.gas_price,
                    value: ActionValue::Transfer(t.value),
                    code: self.state.code(address)?,
                    code_hash: Some( self.state.code_hash(address)?),
                    data: Some(t.data.clone()),
                    call_type: CallType::Call,
                    params_type: vm::ParamsType::Separate,
                };
                let mut out = vec![]; //debug_call here, but fails when unimplemented!()
                (self.call(params, &mut substate, BytesRef::Flexible(&mut out), &mut tracer, &mut vm_tracer), out)
            }
        };
        Ok(self.finalize(t, substate, result, output, tracer.drain(), vm_tracer.drain())?)
    }

    /// continue until next breakpoint
    fn resume_debug_transact<T,V>(&mut self, 
                             t: &SignedTransaction, 
                             options: TransactOptions<T, V>, 
                             pc: usize) -> Result<DebugExecuted<T::Output, V::Output>, ExecutionError>
    {
        unimplemented!();
    }

    
    /// call a contract function with contract params
    /// until 'PC' (program counter) is hit
    fn debug_call<T,V>(&mut self,
                           pc: usize,
                           params: ActionParams,
                           substate: &mut Substate,
                           mut output: BytesRef,
                           tracer: &mut T,
                           vm_tracer: &mut V
    ) -> vm::Result<FinalizationResult> where T: Tracer, V: VMTracer {
        unimplemented!();
    }
    
    /// Execute VM until it hits the program counter
    fn exec_step_vm<T, V>(&mut self, 
                          pc: usize, 
                          schedule: Schedule, 
                          params: ActionParams, 
                          unconfirmed_substate: &mut Substate, 
                          output_policy: OutputPolicy, 
                          tracer: &mut T, 
                          vm_tracer: &mut V) {
        unimplemented!();
    }

}


// serves as example of how executive should be used
#[cfg(test)]
mod tests {
    use ::*;
    use super::*;
    use ethcore::executive::Executive;
    use ethcore::state_db::StateDB;
    use ethcore::BlockChainDB;
    use ethereum_types::{U256};
    use ethcore::state::{State, Backend};
    use ethcore::machine::EthereumMachine;
    use vm::{EnvInfo};
    use kvdb::KeyValueDB;
    use tempdir::TempDir;
    use std::sync::Arc;
    use {blooms_db, journaldb};
    // pub fn new(state: &'a mut State<B>, info: &'a EnvInfo, machine: &'a Machine) -> Self {
    
    // just for tests
    fn make_byzantium_machine(max_depth: usize) -> EthereumMachine {
        let mut machine = ethcore::ethereum::new_byzantium_test_machine();
        machine.set_schedule_creation_rules(Box::new(move |s, _| s.max_depth = max_depth));
        machine
    }

    fn get_params() -> (State<StateDB>, EnvInfo, EthereumMachine) {
        // im assuming this returns a default of some sort for all other arguments
        // in our own configuraiton, we will set the args ourselves
        // for now this suffices
        // let config = parity::Configuration::parse_cli(&["--chain", "dev"]).unwrap();
        struct TestBlockChainDB {
            _blooms_dir: TempDir,
            _trace_blooms_dir: TempDir,
            blooms: blooms_db::Database,
            trace_blooms: blooms_db::Database,
            key_value: Arc<KeyValueDB>,
        }

        impl BlockChainDB for TestBlockChainDB {
            fn key_value(&self) -> &Arc<KeyValueDB> {
                &self.key_value
            }

            fn blooms(&self) -> &blooms_db::Database {
                &self.blooms
            }

            fn trace_blooms(&self) -> &blooms_db::Database {
                &self.trace_blooms
            }
        }

        let blooms_dir = TempDir::new("").unwrap();
        let trace_blooms_dir = TempDir::new("").unwrap();

        let db = TestBlockChainDB {
            blooms: blooms_db::Database::open(blooms_dir.path()).unwrap(),
            trace_blooms: blooms_db::Database::open(trace_blooms_dir.path()).unwrap(),
            _blooms_dir: blooms_dir,
            _trace_blooms_dir: trace_blooms_dir,
            key_value: Arc::new(kvdb_memorydb::create(ethcore::db::NUM_COLUMNS.unwrap()))
        };
        let db: Arc<BlockChainDB> = Arc::new(db);
        let journal_db = journaldb::new(db.key_value().clone(), 
                                        journaldb::Algorithm::EarlyMerge, 
                                        ethcore::db::COL_STATE);
        let mut state_db = StateDB::new(journal_db, 5*1024*1024);
        let mut factories = ethcore::factory::Factories::default(); // factory is private in official parity
        let mut state = State::new(state_db, U256::from(0), factories);
        let info = EnvInfo::default();
        let machine = make_byzantium_machine(0);
        (state, info, machine)
    }

    #[test]
    fn it_should_create_new_executive_extension() {
        let (mut state, info, machine) = get_params();
        Executive::new(&mut state, &info, &machine);
    }

    #[test]
    #[should_panic]
    fn it_should_panic_on_unimplemented() {
        let (mut state, info, machine) = get_params();
        // Executive::new(&mut state, &info, &machine).begin_debug_transact();
        panic!("Definitely not implemented"); // placeholder
    }
}
