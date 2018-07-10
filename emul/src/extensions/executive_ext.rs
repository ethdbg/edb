use ethcore::executive::{Executive, TransactOptions};
use ethcore::executed::{Executed, ExecutionResult};
use ethcore::state::{Backend as StateBackend, State, Substate};
use ethcore::machine::EthereumMachine as Machine;
use ethcore::trace::{self, Tracer, VMTracer};
use ethcore::error::ExecutionError;
use bytes::{Bytes, BytesRef};
use transaction::{Action, SignedTransaction};
use vm::{self, Schedule, ActionParams, EnvInfo};
use evm::{FinalizationResult, Finalize, CallType};
use ethcore_io::LOCAL_STACK_SIZE;
use std::cell::RefCell;
use std::rc::Rc;
use externalities::*;
use instruction_manager::InstructionManager;

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

pub struct ExecutiveExt<'a, B: 'a + StateBackend> {
    info: &'a EnvInfo,
    machine: &'a Machine,
    state: &'a mut State<B>,
    depth: usize,
    static_flag: bool,
}

impl<'a, B: 'a + StateBackend> ExecutiveExt<'a, B> {

    pub fn new(state: &'a mut State<B>, info: &'a EnvInfo, machine: &'a Machine) -> Self {

        ExecutiveExt {
            info, machine, state,
            depth: 0,
            static_flag: false,
        }
    }
   
    /// populate executive from parent properties. Increments executive depth.
    pub fn from_parent(state: &'a mut State<B>, 
                       info: &'a EnvInfo, 
                       machine: &'a Machine, 
                       parent_depth: usize, 
                       static_flag: bool) -> Executive<'a, B> {
        Executive::from_parent(state, info, machine, parent_depth, static_flag)
    }
    
    /// populate executive extension from parent properties. Increments executive depth.
    pub fn from_ext_parent(state: &'a mut State<B>,
                           info: &'a EnvInfo,
                           machine: &'a Machine,
                           parent_depth: usize,
                           static_flag: bool) -> Self {
        ExecutiveExt {
            state, info, machine, static_flag,
            depth: parent_depth+1,
        }
    }

    /// Creates 'Externalities' from 'Executive'
    /// used exclusively in executive_ext, however is a public function on 'Executive' in Parity
    /// Private fields are required to use this function, so no suitable alternative could be made.
    pub fn as_externalities<'any, T, V>(
		&'any mut self,
		origin_info: OriginInfo,
		substate: &'any mut Substate,
		output: OutputPolicy<'any, 'any>,
		tracer: &'any mut T,
		vm_tracer: &'any mut V,
		static_call: bool,
        inst_manager: &'any InstructionManager,
	) -> Externalities<'any, T, V, B> where T: Tracer, V: VMTracer {
        let is_static = self.static_flag || static_call;
        Externalities::new(self.state, self.info, self.machine, self.depth, origin_info, substate, 
                           output, tracer, vm_tracer, is_static, inst_manager)
    }

	pub fn transact<T, V>(&'a mut self, t: &SignedTransaction, options: TransactOptions<T, V>)
		-> Result<Executed<T::Output, V::Output>, ExecutionError> where T: Tracer, V: VMTracer,
    {
        Executive::new(self.state, self.info, self.machine).transact(t, options)
    }

    pub fn transact_virtual<T,V>(&'a mut self, 
                                 t: &SignedTransaction, 
                                 options: TransactOptions<T,V>)
        -> Result<Executed<T::Output, V::Output>, ExecutionError> where T: Tracer, V: VMTracer, 
    {
        Executive::new(self.state, self.info, self.machine).transact_virtual(t, options)
    }

    /// like transact_with_tracer + transact_virtual but with real-time debugging 
    /// functionality. Execute a transaction within the debug context
    fn transact_with_debug() {
        unimplemented!();
    }
    
    /// call a contract function with contract params
    /// until 'PC' (program counter) is hit
    pub fn debug_call<T,V>(&mut self,
                           pc: usize,
                           params: ActionParams,
                           substate: &mut Substate,
                           mut output: BytesRef,
                           tracer: &mut T,
                           vm_tracer: &mut V) /* -> vm::Result<FinalizationResult> where T: Tracer, V: VMTracer */ {
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
                          vm_tracer: &mut V)
       /* -> vm::Result<FinalizationResult> where T: Tracer, V:VMTracer */
    {   
        // LOCAL_STACK_SIZE is a `Cell`
        let local_stack_size = LOCAL_STACK_SIZE.with(|sz| sz.get());
        println!("STACK SIZE {}", local_stack_size);
        
    }


    /* 
    /// Finalize a transaction
    fn finalize<T, V>(&mut self, 
                t: &SignedTransaction,
                mut substate: Substate,
                result: vm::Result<FinalizationResult>,
                output: Bytes,
                trace: Vec<T>,
                vm_trace: Option<V>) 
        -> Result<Executed<T, V>, ExecutionError> 
    {
    
        unimplemented!();   
    }
    */
}

#[cfg(test)]
mod tests {
    use ::*;
 
    // pub fn new(state: &'a mut State<B>, info: &'a EnvInfo, machine: &'a Machine) -> Self {

    #[test]
    fn test_executive_ext() {
        //let db = 
        //let state = State::new
        //ExecutiveExt::new()

    
    }

}
