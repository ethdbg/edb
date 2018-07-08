//! Uses InterpreterExt and InstructionState to run and stop VM
//! Kind of like the Debug version of 'executive.rs' in Ethcore
//! Does not change ethereum state. purely for debugging contracts by themselves with
//! the EVM

pub enum Action {
    StepBack,
    RunUntil,
    Exec,
}

pub enum EmulatorType {
    WithWorldState,
    EvmOnly
}

pub struct Emulator {


}

impl Emulator {

    pub fn new() {

    }

    pub fn run(&self, action: Action) {
    
    }
}
