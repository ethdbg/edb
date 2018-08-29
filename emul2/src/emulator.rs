//! Emulates transaction execution and allows for real-time debugging
use sputnikvm::{ValidTransaction, HeaderParams};
use failure::Error;

/// An action or what should happen for the next step of execution
pub enum Action {
    /// step back one instruction
    StepBack,
    /// step forward one instruction
    StepForward,
    /// RunUntil a PC
    RunUntil(usize),
    /// finish instruction
    Finish,
    /// execute to the end
    Exec,
}


/// Emulation Object
pub struct Emulator;


impl Emulator {
    /// Create a new Emulator
    pub fn new(transaction: ValidTransaction, header: HeaderParams) -> {

    }

    pub fn fire(self, action: Action) -> Result<(), Error> {
        match action {
            StepBack => ,
            StepForward => ,
            RunUntil(usize) => ,
            Finish => ,
            Exec => ,
        }
    }
}
