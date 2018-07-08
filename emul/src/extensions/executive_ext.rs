use ethcore::executive::Executive;
use ethcore::executed::{Executed, ExecutionResult};
use vm::{self, Result, Schedule, ActionParams};
use evm::{FinalizationResult, Finalize, CallType};

pub trait ExecutiveExt {

        /// Finalize a transaction
    fn finalize();
    
    /// like transact_with_tracer + transact_virtual but with real-time debugging 
    /// functionality. Execute a transaction within the debug context
    fn transact_with_debug();

    /// Execute VM until it hits 'pc'
    fn exec_step_vm<T, V>(&mut self, 
                          pc: usize, 
                          schedule: Schedule, 
                          params: ActionParams, 
                          unconfirmed_substate: &mut Substate, 
                          output_policy: OutputPolicy, 
                          tracer: &mut T, 
                          vm_tracer: &mut V) 
        -> vm::Result<FinalizationResult> where T: Tracer, V:VMTracer;

    /// call a contract function with contract params
    ///
    fn debug_call<T, V>(&mut self, 
                        params: ActionParams, 
                        substate: &mut Substate, 
                        mut output: BytesRef, 
                        tracer: &mut T, 
                        vm_tracer: &mut V)
        -> vm::Result<FinalizationResult> where T: Tracer, V: VMTracer;
}


impl ExecutiveExt for Executive {
    fn finalize() -> {
        unimplemented!();
    }
}

