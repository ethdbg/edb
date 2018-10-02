use failure::Error;
use edb_compiler::{Language, CodeFile};
use edb_emul::{emulator::{Emulator, Action}, ValidTransaction, HeaderParams};

use std::path::PathBuf;

pub struct Debugger<T, L> where T: web3::Transport, L: Language {
    file: CodeFile<T, L>,
    emul: Emulator<T>,
    breakpoints: Vec<Breakpoint>,
}

pub type Breakpoint = usize;

impl<T, L> Debugger<T, L> where T: web3::Transport, L: Language {
    pub fn new<T>(path: PathBuf,
                  lang: impl Language,
                  client: web3::Web3<T>,
                  tx: ValidTransaction,
                  block: HeaderParams,
                  )
        -> Result<Self, Error> where T: web3::Transport
    {
        let file = CodeFile::new(lang, path, client.clone())?;
        let emul = Emulator::new(tx, block, client);
        let breakpoints = Vec::new();
        Ok(Self {file, emul, breakpoints })
    }

    /// Begins the program, and runs until it hits a breakpoint
    pub fn run(&self, /*params: Vec<ethabi::Params>*/ ) -> Result<(), Error> {

    }

    /// Runs the transaction to the end, ignoring any breakpoints.
    pub fn run_to_end(&self) -> Result<(), Error> {


    }

    /// Sets a breakpoint at a line number
    pub fn set_breakpoint(&self, line: Breakpoint) -> Result<(), Error> {


    }

    /// Removes a breakpoint if it matches the predicate F
    pub fn remove_breakpoint_by<F>(&self, fun: F) where F: Fn(&Breakpoint) -> bool {

    }

    /// Steps to the next line of execution
    pub fn step(&self, line: usize) -> Result<(), Error> {

    }

    /// Jumps to the next breakpoint in execution
    pub fn next(&self, line: usize) -> Result<(), Error> {


    }

    /// returns the current line of execution
    pub fn current_line(&self) -> Result<String, Error> {

    }

    /// Returns the `count` number of last lines relative to current line of execution
    pub fn last_lines(&self, count: usize) -> Result<Vec<String>, Error> {

    }

    /// Returns the `count` number of next lines relative to the current line of execution
    pub fn next_lines(&self, count: usize) -> Result<Vec<String>, Error> {


    }

    /// Returns the EVM Stack
    pub fn stack(&self) -> Result<(), Error> {


    }
}
