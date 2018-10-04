use failure::Error;
use edb_compiler::{Language, CodeFile};
use edb_emul::{emulator::{Emulator, Action}, ValidTransaction, HeaderParams};
use super::addr_cache::AddressCache;
use std::path::PathBuf;

pub struct Debugger<T, L> where T: web3::Transport, L: Language {
    file: CodeFile<L, T>,
    emul: Emulator<T>,
    breakpoints: Vec<Breakpoint>,
    cache: AddressCache,
    curr_name: String,
}

pub type Breakpoint = usize;

impl<T, L> Debugger<T, L> where T: web3::Transport, L: Language {
    pub fn new(path: PathBuf,
                  lang: L,
                  client: web3::Web3<T>,
                  tx: ValidTransaction,
                  block: HeaderParams,
                  )
        -> Result<Self, Error>
    {
        let cache = AddressCache::new(&client)?;
        let file = CodeFile::new(lang, path, client.clone(), &cache.as_vec().as_slice())?;
        let emul = Emulator::new(tx, block, client);
        let breakpoints = Vec::new();
        let curr_name = String::from("");
        Ok(Self {file, emul, breakpoints, cache, curr_name})
    }

    /// Begins the program, and runs until it hits a breakpoint
    pub fn run(&mut self, name: Option<&str>) -> Result<(), Error> {
        if let Some(b) = self.breakpoints.pop() {
            if name.is_none() {
                panic!("name not specified");
            }
            self.curr_name = name.unwrap().to_string();
            let pos = self.file.position_from_lineno(b, name.unwrap())?;
            self.emul.fire(Action::RunUntil(pos))?;
            Ok(())
        } else {
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
        match self.breakpoints.binary_search(&line) {
            Ok(_) => {} // already inserted
            Err(pos) => self.breakpoints.insert(pos, line)
        };
        Ok(())
    }

    /// Removes a breakpoint if it matches the predicate F
    pub fn remove_breakpoint_by(&mut self, line: Breakpoint) {
        match self.breakpoints.binary_search(&line) {
            Ok(pos) => {self.breakpoints.remove(pos); },
            Err(_) => {} // element not in array
        };
    }

    /// Steps to the next line of execution
    pub fn step(&mut self) -> Result<(), Error> {
        let line = self.file.lineno_from_position(self.emul.offset(), self.curr_name.as_str())?;
        let run_to = self.file.position_from_lineno(line+1, self.curr_name.as_str())?;
        self.emul.fire(Action::RunUntil(run_to))?;
        Ok(())
    }

    /// Jumps to the next breakpoint in execution
    pub fn next(&mut self) -> Result<(), Error> {
        if let Some(b) = self.breakpoints.pop() {
            self.emul.fire(Action::RunUntil(self.file.position_from_lineno(b, self.curr_name.as_str())?))?;
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
    pub fn stack(&self) -> Result<(), Error> {
        unimplemented!();
    }
}
