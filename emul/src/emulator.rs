//! Uses InterpreterExt and InstructionState to run and stop VM
//! Kind of like the Debug version of 'executive.rs' in Ethcore
//! Does not change ethereum state. purely for debugging contracts by themselves with
//! the EVM
use vm;
use evm;
use vm::{Ext, Vm};
use evm::{CostType, Finalize};
use ethcore::executed::ExecutionError;
use evm::interpreter::{Interpreter, SharedCache};
use extensions::interpreter_ext::{InterpreterExt, ExecInfo};
use debug_externalities::ExternalitiesExt;
use std::vec::Vec;
use std::sync::Arc;

#[derive(Debug)]
pub struct FinalizationResult {
    pub finalization_result: Option<Result<evm::FinalizationResult, vm::Error>>,
    pub is_complete: bool,
    pub exec_info: Result<ExecInfo, ExecutionError>,
}


impl FinalizationResult {
    pub fn new(final_res: Option<Result<evm::FinalizationResult, vm::Error>>, 
               is_complete: bool, 
               exec_info: Result<ExecInfo, ExecutionError>
    ) -> Self {
        FinalizationResult {
            finalization_result: final_res,
            is_complete, exec_info,
        }
    }

}

// 0 state is before interpreter did anything
#[derive(Default)]
pub struct InterpreterSnapshots {
    pub states: Vec<Box<InterpreterExt>>,
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
    RunUntil,
    Exec,
}

/*pub enum EmulatorType {
    WithWorldState,
    EvmOnly
}*/
/*
pub trait EDBFinalize {
    fn finalize(self, ext: ExternalitiesExt) -> vm::Result<FinalizationResult>;
}

impl EDBFinalize for vm::Result<ExecInfo> {
    fn finalize<E: ExternalitiesExt>(self, ext: E) -> vm::Result<FinalizationResult> {
        match self {
            Ok(x) => {
                Ok(FinalizationResult {
    /*pub finalization_result: Option<Result<evm::FinalizationResult, vm::Error>>,
    pub is_complete: bool,
    pub exec_info: Result<ExecInfo, ExecutionError>,*/
                    finalization_result: if x.gas_left().is_some() {
                        Some(x.gas_left().unwrap().finalize(ext.externalities()))
                    } else {None}

                })
            
            }
        
        }
    }
}
*/
pub trait VMEmulator {
    fn fire(mut self, action: Action, ext: &mut ExternalitiesExt, pos: usize
    ) -> vm::Result<ExecInfo>;
}

pub struct Emulator<C: CostType + 'static>(Interpreter<C>);

impl<C: CostType + 'static> VMEmulator for Emulator<C> {
    /// Fire
    fn fire(mut self, action: Action, ext: &mut ExternalitiesExt, pos: usize
    ) -> vm::Result<ExecInfo> {

        match action {
            Action::StepBack => self.0.step_back(ext),
            Action::RunUntil => self.0.run_code_until(ext, pos),
            Action::Exec => self.0.run(ext.externalities()),
            _ => panic!("Action not found")
        }
    }
}

impl<Cost: CostType> Emulator<Cost> {
    pub fn new(params: vm::ActionParams, cache: Arc<SharedCache>, ext: &Ext) -> Self {
        Emulator(Interpreter::new(params, cache, ext).unwrap())
    }
}
