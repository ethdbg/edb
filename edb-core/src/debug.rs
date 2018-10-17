use failure::Error;
use std::path::PathBuf;
use log::*;
use edb_compiler::{Language, CodeFile, AbstractFunction};
use edb_emul::{emulator::{Emulator, Action}, ValidTransaction, HeaderParams};
use super::addr_cache::AddressCache;
use super::err::EvmError;

pub struct Debugger<T, L> where T: web3::Transport, L: Language {
    file: CodeFile<L, T>,
    emul: Emulator<T>,
    breakpoints: Vec<Breakpoint>,
    // cache: AddressCache,
    curr_name: String,
}

pub type Breakpoint = usize;

impl<T, L> Debugger<T, L> where T: web3::Transport, L: Language {

    pub fn new(path: PathBuf,
                  lang: L,
                  client: web3::Web3<T>,
                  tx: ValidTransaction,
                  block: HeaderParams,
                  contract_name: &str
                  )
        -> Result<Self, Error>
    {
        let cache = AddressCache::new(&client)?;
        let file = CodeFile::new(lang, path, client.clone(), &cache.as_vec().as_slice())?;
        let emul = Emulator::new(tx, block, client);
        let breakpoints = Vec::new();
        let curr_name = String::from(contract_name);
        Ok(Self {file, emul, breakpoints, curr_name})
    }

    /// Begins the program, and runs until it hits a breakpoint
    pub fn run(&mut self) -> Result<(), Error> {
        if let Some(b) = self.breakpoints.pop() {
            let pos = self.file.unique_opcode_pos(b, self.curr_name.as_str())?;
            self.emul.fire(Action::RunUntil(pos))?;
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
    pub fn step(&mut self) -> Result<(), Error> {
        let contract = self.file.find_contract(self.curr_name.as_str())?;
        debug!("Finding Line from position {}, and contract {}", self.emul.offset(), self.curr_name.as_str());
        let current_line = self.file.lineno_from_opcode_pos(self.emul.offset(), self.curr_name.as_str())?;
        let offset = self.file.char_pos_from_lineno(current_line, self.curr_name.as_str())?;
        trace!("Current Instruction: {:?}", self.file.opcode_pos_from_lineno(current_line, self.emul.offset(), self.curr_name.as_str())?);

        let mut func_range = None;
        contract.file().find_function(offset, &mut |func| func_range = Some(func.expect("Location").location()));
        let (func_start, func_end) = func_range.expect("Range");

        let file = &self.file; let curr_name = self.curr_name.as_str();
        self.emul.step_until(|mach| { // step until opcode reaches a line that is not the current line
            let line = file.lineno_from_opcode_pos(mach.pc().opcode_position(), curr_name)?;
            trace!("Line found step_until(): {}, current line: {}", line, current_line);
            let char_offset = file.char_pos_from_lineno(line, curr_name)?;

            Ok(line != current_line && char_offset >= func_start && char_offset <= func_end)
        })?;

        self.emul.read_raw(|vm| {
            if let Some(mach) = vm.current_machine() {
                trace!("PC: {}", mach.pc().position());
                trace!("Opcode Position: {}", mach.pc().opcode_position());
                trace!("Next Opcode: {:?}", mach.pc().peek_opcode().expect("Opcode"));
                Ok(())
            } else {
                Ok(())
            }
        })?;
        Ok(())
    }

    /// Jumps to the next breakpoint in execution
    pub fn next(&mut self) -> Result<(), Error> {
        if let Some(b) = self.breakpoints.pop() {
            self.emul.fire(Action::RunUntil(self.file.opcode_pos_from_lineno(b, self.emul.offset(), self.curr_name.as_str())?))?;
        } else {
            self.emul.fire(Action::Exec)?;
        }
        Ok(())
    }

    /// returns the current line of execution
    pub fn current_line(&self) -> Result<(usize, String), Error> {
        self.file.current_line(self.emul.offset(), self.curr_name.as_str())
    }

    /// Returns the `count` number of last lines relative to current line of execution
    pub fn last_lines(&self,count: usize) -> Result<Vec<(usize, String)>, Error> {
        self.file.last_lines(self.emul.offset(), count, self.curr_name.as_str())
    }

    /// Returns the `count` number of next lines relative to the current line of execution
    pub fn next_lines(&self, count: usize) -> Result<Vec<(usize, String)>, Error> {
        self.file.next_lines(self.emul.offset(), count, self.curr_name.as_str())
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
        });

        Ok(stack_vec)
    }
}
