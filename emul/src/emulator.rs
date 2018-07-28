//! Uses InterpreterExt and InstructionState to run and stop VM
//! Kind of like the Debug version of 'executive.rs' in Ethcore
//! Does not change ethereum state. purely for debugging contracts by themselves with
//! the EVM
use vm::Schedule;
use evm::CostType;
use evm::interpreter::{Interpreter, SharedCache};
use std::vec::Vec;
use std::sync::Arc;
use crate::err::Result;
use crate::extensions::{InterpreterExt, ExecInfo};
use crate::externalities::{ExternalitiesExt};

// 0 state is before interpreter did anything
#[derive(Default)]
pub struct InterpreterSnapshots {
    pub states: Vec<Box<dyn InterpreterExt + Send>>,
}

impl InterpreterSnapshots {
    pub fn new() -> Self {

        InterpreterSnapshots {
            states: Vec::new()
        }
    }
}

pub enum Action {
    StepBack,
    StepForward,
    RunUntil(usize),
    Finish,
    Exec,
}

/*pub enum EmulatorType {
    WithWorldState,
    EvmOnly
}*/

pub trait VMEmulator {
    fn fire(&mut self, action: &Action, ext: &mut dyn ExternalitiesExt) -> Result<ExecInfo>;
}

pub struct Emulator<C: CostType + Send + 'static>(Interpreter<C>);

impl<C: CostType + Send + 'static> VMEmulator for Emulator<C> {
    /// Fire
    // needs to be a Box<Self> because of mutations inherant to`self` in step_back()
    fn fire(&mut self, action: &Action, ext: &mut dyn ExternalitiesExt) -> Result<ExecInfo> {

        match action {
            Action::StepBack => self.0.step_back(ext),
            Action::RunUntil(pc) => self.0.run_code_until(ext, *pc),
            Action::Exec => self.0.run(ext.externalities()),
            _ => panic!("Action not found")
        }
    }
}

impl<Cost: CostType + Send> Emulator<Cost> {
    pub fn new(params: vm::ActionParams, cache: Arc<SharedCache>, schedule: &Schedule, depth: usize) -> Self {
        Emulator(Interpreter::new(params, cache, schedule, depth))
    }
}
