use crossbeam;
use evm;
use ethcore::executive::{Executive, contract_address, TransactOptions, STACK_SIZE_PER_DEPTH, STACK_SIZE_ENTRY_OVERHEAD};
use ethcore::executed::{Executed, ExecutionError};
use ethcore::externalities::*;
use ethcore::state::{Backend as StateBackend, Substate, CleanupMode};
use ethcore::trace::{Tracer, VMTracer, FlatTrace, VMTrace};
use ethcore_io::LOCAL_STACK_SIZE;
use vm::{self, Schedule, ActionParams, ActionValue, CleanDustMode, Ext, ReturnData, GasLeft};
use evm::{Finalize, CallType};
use ethereum_types::{U256, U512};
use bytes::{Bytes, BytesRef};
use transaction::{Action as TxAction, SignedTransaction};
use std::sync::Arc;

use emulator::{Action, Emulator, VMEmulator, FinalizationResult, EDBFinalize};
use externalities::ExternalitiesExt;
use extensions::{ExecInfo, FactoryExt};
use err::{Result, Error};

/* any functions here should be taken directly from parity; no modification. We only split off
 * functions and put them into executive.rs if they need to be modified to fit our needs 
 * other then error types and extender functions (like as_dbg_externalities)
 * this way, it will be easier to keep up with parity changes
 * these basically act as 'shims', intercepting the flow of the functions, so that EDB can 
 * modify what the EVM does
 * */

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

#[derive(Debug, Clone)]
pub struct DebugExecuted<T = FlatTrace, V = VMTrace> {
    pub executed: Option<Executed<T,V>>,
    is_complete: bool,
    exec_info: ExecInfo,
}

pub trait ExecutiveExt<'a, B: 'a + StateBackend> {
   
    fn as_dbg_externalities<'any, T, V>(&'any mut self,
                                        origin_info: OriginInfo,
                                        substate: &'any mut Substate,
                                        output: OutputPolicy<'any, 'any>,
                                        tracer: &'any mut T,
                                        vm_tracer: &'any mut V,
                                        static_call: bool
    ) -> DebugExt<'any, T, V, B> where T: Tracer, V: VMTracer;
        
    
    /// like transact_with_tracer + transact_virtual but with real-time debugging 
    /// functionality. Execute a transaction within the debug context
    /// pc = Program Counter (where to stop execution)
    // Prefer enum over two different functions
    fn _transact_debug<T, V>(&'a mut self, 
                           t: &SignedTransaction,
                           options: TransactOptions<T,V>,
    ) -> Result<(sender: Address, init_gas: U256, tracer: T, vm_tracer: V, substate: &mut Substate)> 
        where T: Tracer,
              V: VMTracer;
    /// call a contract function with contract params
    /// until 'PC' (program counter) is hit
    fn _debug_call<T,V>( &mut self,
                        params: ActionParams,
                        substate: &mut Substate,
                        output: BytesRef,
                        tracer: &mut T,
                        vm_tracer: &mut V
    ) -> Result<evm::FinalizationResult> where T: Tracer, V: VMTracer;
    
    /// Execute VM until it hits the program counter
    fn _exec_step_vm<T, V>(&mut self, 
                          schedule: Schedule, 
                          params: ActionParams, 
                          unconfirmed_substate: &mut Substate, 
                          output_policy: OutputPolicy, 
                          tracer: &mut T, 
                          vm_tracer: &mut V
    ) -> Result<FinalizationResult> where T: Tracer, V: VMTracer;
}

// TODO: add enum type that allows a config option to choose whether to execute with or without 
// validation
impl<'a, B: 'a + StateBackend> ExecutiveExt<'a, B> for Executive<'a, B> {

    fn as_dbg_externalities<'any, T, V>(&'any mut self,
                                        origin_info: OriginInfo,
                                        substate: &'any mut Substate,
                                        output: OutputPolicy<'any, 'any>,
                                        tracer: &'any mut T,
                                        vm_tracer: &'any mut V,
                                        static_call: bool
    ) -> DebugExt<'any, T, V, B> where T: Tracer, V: VMTracer {
        let is_static = self.static_flag || static_call;
        DebugExt::new(self.state, self.info, self.machine, self.depth, origin_info, substate,
                      output, tracer, vm_tracer, is_static)
    }
 

    /// like transact_with_tracer + transact_virtual but with real-time debugging 
    /// functionality. Execute a transaction within the debug context
    fn _transact_debug<T, V>(&'a mut self, t: &SignedTransaction, options: 
                                  TransactOptions<T,V>,
    ) -> Result<(sender: Address, init_gas: U256, tracer: T, vm_tracer: V, substate: &mut Substate)>
        where T: Tracer, V: VMTracer
    {   
        let output_from_create = options.output_from_init_contract;
        let mut tracer = options.tracer;
        let mut vm_tracer = options.vm_tracer;

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
            return Err(Error::from(ExecutionError::NotEnoughBaseGas { required: base_gas_required, got: t.gas })); 
        }

        if !t.is_unsigned() && options.check_nonce && schedule.kill_dust != CleanDustMode::Off && 
            !self.state.exists(&sender)? {
                return Err(Error::from(ExecutionError::SenderMustExist));
        }
 
        let init_gas = t.gas - base_gas_required;
        
        // tx nonce validation
        if options.check_nonce && t.nonce != nonce {
            return Err(Error::from(ExecutionError::InvalidNonce { expected: nonce, got: t.nonce }));
        }

        // validate if tx fits into block
        if self.info.gas_used + t.gas > self.info.gas_limit {
            return Err(Error::from(ExecutionError::BlockGasLimitReached {
                gas_limit: self.info.gas_limit,
                gas_used: self.info.gas_used,
                gas: t.gas
            }));
        }
        
        let balance = self.state.balance(&sender)?;
        let gas_cost = t.gas.full_mul(t.gas_price);
        let total_cost = U512::from(t.value) + gas_cost;
        
        // validate if user can afford tx
        let balance512 = U512::from(balance);
        if balance512 < total_cost {
            return Err(Error::from(ExecutionError::NotEnoughCash { required: total_cost, got: balance512 }));
        }

        let mut substate = Substate::new();

        if !schedule.eip86 || !t.is_unsigned() {
            self.state.inc_nonce(&sender)?;
        }
    
        self.state.sub_balance(&sender, 
                               &U256::from(gas_cost), 
                               &mut substate.to_cleanup_mode(&schedule))?;
        Ok((sender, init_gas, substate, tracer, vm_tracer))
    }

    /// call a contract function with contract params
    /// until 'PC' (program counter) is hit
    fn _debug_call<T,V>( &mut self,
                        params: ActionParams,
                        substate: &mut Substate,
                        mut output: BytesRef,
                        tracer: &mut T,
                        vm_tracer: &mut V
    // ) -> Result<evm::FinalizationResult> where T: Tracer, V: VMTracer {
    ) -> Option<(schedule: Schedule, 
                 params: ActionParams, output_policy: OutputPolicy, 
                 tracer: &mut T, vm_tracer: &mut V)> 
        where T: Tracer, V: VMTracer {

        // skip builtin contracts
        // TODO: Add builtin contracts
        trace!("Executive::call(params={:?}) self.env_info={:?}, static={}", 
               params, self.info, self.static_flag);
        if (params.call_type == CallType::StaticCall 
                             || ((params.call_type == CallType::Call) && self.static_flag)) 
            && params.value.value() > 0.into() 
        {
            return Err(Error::from(vm::Error::MutableCallInStaticContext));
        }
        self.state.checkpoint();
        let schedule = self.machine.schedule(self.info.number);
        if let ActionValue::Transfer(val) = params.value {
            self.state.transfer_balance(&params.sender, &params.address, &val, 
                                        substate.to_cleanup_mode(&schedule))?;
        }

        /* skip builtins (for now) */
        let trace_info = tracer.prepare_trace_call(&params);
        let mut trace_output = tracer.prepare_trace_output();
        let mut subtracer = tracer.subtracer();

        let gas = params.gas;

        if params.code.is_some() {
            let mut unconfirmed_substate = Substate::new();
            let mut subvmtracer = vm_tracer.prepare_subtrace(params.code.as_ref().expect("scope is conditional on params.code.is_some(); qed"));
            Some(schedule, 
                 params, 
                 &mut unconfirmed_substate, 
                 OutputPolicy::Return(output, trace_output.as_mut()),
                 &mut subtracer, 
                 &mut subvmtracer)
        } else { None }
    }

    fn _end_call<T, V>(&mut self, 
                      tracer: &mut T,
                      vm_tracer: &mut V,
                      trace_output: Option<Bytes>,
                      trace_info: Option<Call>,
                      subtracer: T,
                      subvmtracer: V,
                      exec_result: Result<evm::FinalizationResult>,
                      substate: &mut Substate,
                      unconfirmed_substate: &mut Substate,
                      gas: U256,
        ) -> Result<evm::FinalizationResult> {

        vm_tracer.done_subtrace(subvmtracer);
        trace!(target: "executive", "res={:?}", res);
        let traces = subtracer.drain();
        /* put this out to seperate function where 'FinalizationResult' is guaranteed to be a
         * Result<FinalizationResult> type. 
         * requires Observer events
         * make event 'end' which returns Result<FinalizationResult>
         * seperate FinalizationResult and ExecInfo
         * */

        match exec_result {
            Ok(ref res) if res.apply_state => tracer.trace_call(trace_info, gas - res.gas_left,
                                                                trace_output,traces),
            Ok(_) => tracer.trace_failed_call(trace_info, traces, vm::Error::Reverted.into()),
            Err(ref e) => tracer.trace_failed_call(trace_info, traces, e.into()),
        };
        trace!(target: "executive", "substate={:?}; uncomfirmed_substate={:?} \n", substate, unconfirmed_substate);
        self.enact_result(&exec_result, substate, unconfirmed_substate);
        trace!(target: "executive", "enacted: substate={:?} \n", substate);
        Ok(exec_result)
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
        //Executive::new(&mut state, &info, &machine).begin_debug_transact();
        panic!("Definitely not implemented"); // placeholder
    }
}
