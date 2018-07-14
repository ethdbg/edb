use vm;
use ethcore::externalities::{Externalities, OriginInfo, OutputPolicy};
use ethcore::state::{Backend as StateBackend, State, Substate};
use ethcore::machine::EthereumMachine as Machine;
use extensions::interpreter_ext::InterpreterExt;
use vm::{EnvInfo, Schedule, Ext};
use evm::interpreter::Interpreter;
use evm::CostType;
use ethcore::trace::{Tracer, VMTracer};
use emulator::InterpreterSnapshots;
use std::any::Any;

//TODO move debug_externalities to extensions under externalities_ext;
//will require refactoring of use's

pub struct DebugExt<'a, T: 'a, V: 'a, B: 'a> {
    pub externalities: Externalities<'a, T, V, B>,
    snapshots: InterpreterSnapshots,
}
pub trait ExternalitiesExt<'a> {
    fn push_snapshot(&mut self, interpreter: Box<InterpreterExt>);
    fn step_back(&mut self) -> Box<InterpreterExt>;
    fn snapshots_len(&self) -> usize;
    fn externalities(&mut self) -> &mut vm::Ext;
    // fn consume_ext(self) -> vm::Ext;
}

impl<'a, T: 'a, V: 'a, B: 'a> DebugExt<'a, T, V, B> 
    where T: Tracer,
          V: VMTracer,
          B: StateBackend,
{   
    pub fn new( state: &'a mut  State<B>,
                env_info: &'a EnvInfo,
                machine: &'a Machine,
                depth: usize,
                origin_info: OriginInfo,
                substate: &'a mut Substate,
                output: OutputPolicy<'a, 'a>,
                tracer: &'a mut T,
                vm_tracer: &'a mut V,
                static_flag: bool
    ) -> Self {
        DebugExt {
            externalities: Externalities::new(state, env_info, machine, depth, origin_info, 
                                              substate, output, tracer, vm_tracer, static_flag),
            snapshots: InterpreterSnapshots::new()
        }
    }
}

impl<'a, T: 'a, V: 'a, B: 'a> ExternalitiesExt<'a> for DebugExt<'a, T, V, B> 
    where T: Tracer,
          V: VMTracer,
          B: StateBackend,
{
    fn push_snapshot(&mut self, interpreter: Box<InterpreterExt>) {
        self.snapshots.states.push(interpreter);
    }

    fn step_back(&mut self) -> Box<InterpreterExt> {
         if self.snapshots.states.len() <= 1 {
            self.snapshots.states.pop().unwrap()
        } else {
            // pop latest step
            self.snapshots.states.pop();
            // state = one step back
            self.snapshots.states.pop().unwrap()
        }
    }

    fn snapshots_len(&self) -> usize {
        self.snapshots.states.len()
    }

    fn externalities(&mut self) -> &mut Ext {
        &mut self.externalities
    }

    /*fn consume_ext(self) -> Externalities<'a, T, V, B> {
        self.externalities
    }*/
}

