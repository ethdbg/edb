use {vm, err};
use ethcore::executive::Executive;
use ethcore::executed::{Executed, ExecutionError};
use ethcore::externalities::*;
use ethcore::state::{Backend as StateBackend, Substate, CleanupMode};
use ethcore::trace::{Tracer, VMTracer, FlatTrace, VMTrace};
use vm::{ActionParams, ActionValue, CleanDustMode};
use evm::CallType;
use ethereum_types::{U256, U512, Address};
use transaction::{SignedTransaction};
use externalities::DebugExt;
use extensions::ExecInfo;
use err::Error;
use utils::DebugReturn;
use std::sync::Arc;

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
    fn _transact_debug(&mut self, 
                           t: Arc<SignedTransaction>,
                           check_nonce: bool,
    ) -> err::Result<(Address, U256, U512)>;

    /// call a contract function with contract params
    /// until 'PC' (program counter) is hit
    fn _debug_call<T, V>( &mut self,
                        params: &ActionParams,
                        substate: &mut Substate,
                        tracer: &mut T,
                        vm_tracer: &mut V
    // find a better way to do these return arguments 
    ) -> vm::Result<DebugReturn<T, V>> where T: Tracer, V: VMTracer;
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
    fn _transact_debug(&mut self, t: Arc<SignedTransaction>, check_nonce: bool) 
        -> err::Result<(Address, U256, U512)> {  
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

        if !t.is_unsigned() && check_nonce && schedule.kill_dust != CleanDustMode::Off && 
            !self.state.exists(&sender)? {
                return Err(Error::from(ExecutionError::SenderMustExist));
        }
 
        let init_gas = t.gas - base_gas_required;
        
        // tx nonce validation
        if check_nonce && t.nonce != nonce {
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

        Ok((sender, init_gas, gas_cost))
    }

    /// call a contract function with contract params
    /// until 'PC' (program counter) is hit
    fn _debug_call<T,V>( &mut self,
                        params: &ActionParams,
                        substate: &mut Substate,
                        tracer: &mut T,
                        vm_tracer: &mut V
    // ) -> Result<evm::FinalizationResult> where T: Tracer, V: VMTracer {
    ) -> vm::Result<DebugReturn<T, V>> where T: Tracer, V: VMTracer {
        
        // skip builtin contracts
        // TODO: Add builtin contracts
        trace!("Executive::call(params={:?}) self.env_info={:?}, static={}", 
               params, self.info, self.static_flag);
        if (params.call_type == CallType::StaticCall 
                             || ((params.call_type == CallType::Call) && self.static_flag)) 
            && params.value.value() > 0.into() 
        {
            return Err(vm::Error::MutableCallInStaticContext);
        }
        self.state.checkpoint();
        let schedule = self.machine.schedule(self.info.number);
        if let ActionValue::Transfer(val) = params.value {
            self.state.transfer_balance(&params.sender, &params.address, &val, 
                                        substate.to_cleanup_mode(&schedule))?;
        }

        /* skip builtins (for now) */
        let trace_info = tracer.prepare_trace_call(&params);
        let trace_output = tracer.prepare_trace_output();
        let subtracer = tracer.subtracer();

        if params.code.is_some() {
            let unconfirmed_substate = Substate::new();
            let subvmtracer = vm_tracer.prepare_subtrace(params.code.as_ref().expect("scope is conditional on params.code.is_some(); qed"));
            Ok(DebugReturn {
                        schedule: Some(schedule),
                        unconfirmed_substate: Some(unconfirmed_substate),
                        trace_output,
                        trace_info,
                        subtracer: Some(subtracer),
                        subvmtracer: Some(subvmtracer),
                        is_code: true
                })
        } else { Ok(
                    DebugReturn { 
                        trace_output, trace_info,
                        schedule: None,
                        unconfirmed_substate: None,
                        subtracer: None,
                        subvmtracer: None,
                        is_code: false
                    }
                )
        }
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
