//! Core debugging functions

use std::{
    path::PathBuf,
    collections::HashMap
};

use failure::Error;
use log::*;
use sputnikvm::Memory;

use edb_compiler::{CodeFile, AbstractFunction, CompiledFiles};
use edb_emul::{emulator::{Emulator, Action}, ValidTransaction, HeaderParams};
use super::err::EvmError;

pub struct Debugger<T> where T: web3::Transport {
    file: CodeFile,
    emul: Emulator<T>,
    breakpoints: Vec<Breakpoint>,
    curr_name: String,
}

pub type Breakpoint = usize;

impl<T> Debugger<T> where T: web3::Transport {

    pub fn new(path: PathBuf,
               files: CompiledFiles,
               client: web3::Web3<T>,
               tx: ValidTransaction,
               block: HeaderParams,
               contract_name: &str
                )
        -> Result<Self, Error>
    {
        let file = CodeFile::new(files, path)?;
        let emul = Emulator::new(tx, block, client);
        let breakpoints = Vec::new();
        let curr_name = String::from(contract_name);
        Ok(Self {file, emul, breakpoints, curr_name})
    }
    
    // TODO finish
    /* set emulator and TX
    pub fn set(tx: ValidTransaction, block: HeaderParams) -> Result<(), Error> {
        unimplemented!();
    }
    */

    /// Begins the program, and runs until it hits a breakpoint
    pub fn run(&mut self) -> Result<(), Error> {
        self.emul.fire(Action::StepForward)?;
        if let Some(b) = self.breakpoints.pop() {
            self.step_loop(|line| *line == b)?;
            Ok(())
        } else { // if no breakpoints, just execute the contract
            self.emul.fire(Action::Exec)?;
            Ok(())
        }
    }

    /// Runs the transaction to the end, ignoring any breakpoints.
    pub fn run_to_end(&mut self) -> Result<(), Error> {
        self.emul.fire(Action::Exec)?;
        Ok(())
    }

    /// Sets a breakpoint at a line number
    pub fn set_breakpoint(&mut self, line: Breakpoint) -> Result<(), Error> {
        if self.file.unique_exists(line, self.curr_name.as_str())? {
            match self.breakpoints.binary_search(&line) {
                Ok(_) => {} // already inserted
                Err(pos) => self.breakpoints.insert(pos, line)
            };
        }
        Ok(())
    }

    /// Removes a breakpoint
    pub fn remove_breakpoint(&mut self, line: Breakpoint) {
        match self.breakpoints.binary_search(&line) {
            Ok(pos) => {self.breakpoints.remove(pos); },
            Err(_) => {} // element not in array
        };
    }

    /// Steps to the next line of execution
    pub fn step_forward(&mut self) -> Result<(), Error> {
        // let contract = self.file.find_contract(self.curr_name.as_str())?;
        debug!("Finding Line from position {}, and contract {}", self.emul.instruction(), self.curr_name.as_str());
        let current_line = self.file.lineno_from_opcode_pos(self.emul.instruction(), self.curr_name.as_str())?;
        debug!("Current line: {}", current_line);
        // let offset = self.file.char_pos_from_lineno(current_line, self.curr_name.as_str())?;

/*        let function = contract.file().find_function(&mut |func| {
            let  (start, end) = func.location();
            if start <= offset && end >= offset {
                return true;
            }
            false
        }).expect("Could not find function");
        let (func_start, func_end) = function.location;
        */

        self.step_loop(|line| *line != current_line)?;
        Ok(())
    }

    fn step_loop<F>(&mut self, fun: F) -> Result<(), Error>
    where
        F: Fn(&usize) -> bool
    {
        'step: loop {
            let line = self.file.lineno_from_opcode_pos(self.emul.instruction(), self.curr_name.as_str())?;
            // let char_offset = self.file.char_pos_from_lineno(line, self.curr_name.as_str())?;
            info!("Current line: {}", line);
            if fun(&line) || self.emul.finished() {
                break 'step;
            } else {
                self.emul.fire(Action::StepForward)?;
            }
        }
        Ok(())
    }

    /// Jumps to the next breakpoint in execution
    pub fn next(&mut self) -> Result<(), Error> {
        if let Some(b) = self.breakpoints.pop() {
            self.emul.fire(Action::RunUntil(self.file.opcode_pos_from_lineno(b, self.emul.instruction(), self.curr_name.as_str())?))?;
        } else {
            self.emul.fire(Action::Exec)?;
        }
        Ok(())
    }

    /// returns the current line of execution
    pub fn current_line(&self) -> Result<(usize, String), Error> {
        self.file.current_line(self.emul.instruction(), self.curr_name.as_str())
    }

    /// Returns the `count` number of last lines relative to current line of execution
    pub fn last_lines(&self,count: usize) -> Result<Vec<(usize, String)>, Error> {
        self.file.last_lines(self.emul.instruction(), count, self.curr_name.as_str())
    }

    /// Returns the `count` number of next lines relative to the current line of execution
    pub fn next_lines(&self, count: usize) -> Result<Vec<(usize, String)>, Error> {
        self.file.next_lines(self.emul.instruction(), count, self.curr_name.as_str())
    }

    /// Chain another transaction on the VM, optionally with a new blockheader
    /// executes with previous state of VM
    pub fn chain(&mut self, tx: ValidTransaction, block: Option<HeaderParams>) {
        self.emul.chain(tx, block)
    }

    /// get the return value of the function
    pub fn output(&self) -> Vec<u8> {
        self.emul.output()
    }

    /// Returns the EVM Stack
    pub fn stack(&self) -> Result<Vec<ethereum_types::U256>, Error> {
        let mut stack_vec = Vec::new();

        self.emul.read_raw(|vm| {
            let state = vm.current_state().ok_or(EvmError::NotInitialized)?;
            for i in 0..state.stack.len() {
                stack_vec.push({
                    let item = state.stack.peek(i).map_err(|e| EvmError::from(e))?;
                    ethereum_types::U256((item.0).0)
                });
            }
            Ok(())
        })?;

        Ok(stack_vec)
    }

    /// returns evm memory
    pub fn memory(&self) -> Result<Vec<bigint::M256>, Error> {
        let mut mem_vec = Vec::new();
        let mem = self.emul.memory();
        for i in 0..mem.len() {
            mem_vec.push(mem.read(i.into()));
        }
        Ok(mem_vec)
    }

    pub fn storage(&self) -> Option<HashMap<bigint::U256, bigint::M256>> {
        self.emul.storage()
    }
}
