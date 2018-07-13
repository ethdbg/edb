//! Uses InterpreterExt and InstructionState to run and stop VM
//! Kind of like the Debug version of 'executive.rs' in Ethcore
//! Does not change ethereum state. purely for debugging contracts by themselves with
//! the EVM
use vm::{Ext, Vm};
use vm;
use evm::{CostType};
use evm::interpreter::{Interpreter, SharedCache};
use extensions::interpreter_ext::InterpreterExt;
use std::vec::Vec;
use std::sync::Arc;


// 0 state is before interpreter did anything
#[derive(Default)]
pub struct InterpreterSnapshots<Cost: CostType> {
    pub states: Vec<Interpreter<Cost>>,
}

impl<Cost: CostType> InterpreterSnapshots<Cost> {
    pub fn new() -> Self {

        InterpreterSnapshots {
            states: Vec::new()
        }
    }
}

pub enum Action {
    StepBack,
    RunUntil,
    Exec,
}

/*pub enum EmulatorType {
    WithWorldState,
    EvmOnly
}*/

pub struct Emulator<Cost: CostType> {
    interpreter: Interpreter<Cost>,
    snapshots: InterpreterSnapshots<Cost>,
}

impl<Cost: CostType> Emulator<Cost> {

    pub fn new(params: vm::ActionParams, cache: Arc<SharedCache>, ext: &Ext) -> Self {
        Emulator {
            interpreter: Interpreter::new(params, cache, ext).unwrap(),
            snapshots: InterpreterSnapshots::new(),
        }
    }
    
    /// Fire
    pub fn fire(mut self, action: Action, ext: &mut Ext, pos: usize) {

        match action {
            Action::StepBack => {
                self.interpreter = self.interpreter.step_back(&mut self.snapshots);
            }
            Action::RunUntil => {self.interpreter.run_code_until(ext, pos, &mut self.snapshots);},
            Action::Exec => {self.interpreter.exec(ext);},
            _ => panic!("Action not found")
        }
    }
}
