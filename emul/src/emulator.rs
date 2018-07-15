//! Uses InterpreterExt and InstructionState to run and stop VM
//! Kind of like the Debug version of 'executive.rs' in Ethcore
//! Does not change ethereum state. purely for debugging contracts by themselves with
//! the EVM
use vm;
use evm;
use vm::{Ext};
use evm::{CostType, Finalize};
use ethcore::executed::ExecutionError;
use evm::interpreter::{Interpreter, SharedCache};
use extensions::interpreter_ext::{InterpreterExt, ExecInfo};
use debug_externalities::{ConsumeExt, ExternalitiesExt};
use std::vec::Vec;
use std::sync::Arc;
use ethcore::trace::{Tracer, VMTracer};
use ethcore::state::Backend as StateBackend;


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

pub trait EDBFinalize<'a, T: 'a, V: 'a, B: 'a> {
    fn finalize<E>(self, ext: E) -> vm::Result<FinalizationResult>
        where T: Tracer,
              V: VMTracer,
              B: StateBackend,
              E: ExternalitiesExt + ConsumeExt<'a, T, V, B>;
}

impl<'a, T: 'a, V: 'a, B: 'a> EDBFinalize<'a, T, V, B> for vm::Result<ExecInfo> {
    fn finalize<E>(self, ext: E) -> vm::Result<FinalizationResult> 
        where T: Tracer,
              V: VMTracer,
              B: StateBackend,
              E: ExternalitiesExt + ConsumeExt<'a, T, V, B>
    {
        match self {
            Ok(x) => {
                Ok(FinalizationResult {
                    finalization_result: if x.gas_left().is_some() {
                        Some(x.gas_left().to_owned().unwrap().finalize(ext.consume()))
                    } else {None},
                    is_complete: if x.gas_left().is_some() {true} else {false},
                    exec_info: Ok(x)
                })
            },
            Err(e) => Err(vm::Error::Internal(e.to_string()))
        }
    }
}

pub trait VMEmulator {
    fn fire(self, action: Action, ext: &mut ExternalitiesExt, pos: usize
    ) -> vm::Result<ExecInfo>;
}

pub struct Emulator<C: CostType + Send + 'static>(Interpreter<C>);

impl<C: CostType + Send + 'static> VMEmulator for Emulator<C> {
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

impl<Cost: CostType + Send> Emulator<Cost> {
    pub fn new(params: vm::ActionParams, cache: Arc<SharedCache>, ext: &Ext) -> Self {
        Emulator(Interpreter::new(params, cache, ext).unwrap())
    }
}
