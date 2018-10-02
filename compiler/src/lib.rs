#![recursion_limit="128"]
#![feature(test, slice_concat_ext)]
//! Interfaces 'compiler' modules must implement
//! Three Main Traits:
//!     - Contract: The Contract interface that represents *one* Contract
//!     - Language: Represents the Language Type, and everything that can be done with a Language
//!         - Source Mapping
//!         - Walking the AST
//!         - Compiling
//!         - etc
//!     - SourceMap: A trait every 'Source Mapping' Struct must implement
//!         - Mapping Byte offsets to positions in file
//!         - getting information about the mapping from Source to Bytecode, Source to AST
//!     - `CodeFile` Struct accepts a type that implements Language and SourceMap
mod err;
mod types;
mod contract;
pub mod map;
mod code_file;

pub mod solidity;
// pub mod vyper;

pub use self::code_file::CodeFile;
pub use self::contract::{Contract, ContractFile};

use std::{path::PathBuf, rc::Rc};
use web3::Transport;
use failure::Error;
extern crate test;


/// The Source File of a specific language
pub trait Language {
    /// Compiles Source Code File into a Vector of Contract Files
    fn compile<T>(&self, path: PathBuf, client: &web3::api::Eth<T>)
        -> Result<(Vec<Rc<ContractFile>>, Vec<Contract<T>>), Error> where T: Transport;
}

/// Represents a Line - Line number and String (0-indexed)
pub type Line = (usize, String);
pub type LineNo = usize;
pub type Offset = usize;
/// Represents a Source Map
pub trait SourceMap {

    /// Get the instruction offset from a line number in the Source Code
    /// Optional File - if not specified, takes first file in index
    fn position_from_lineno(&self, lineno: usize) -> Result<Offset, Error>;

    /// The reverse of `position_from_lineno`
    fn lineno_from_position(&self, offset: usize) -> Result<LineNo, Error>;

    /// Get a line mapping (line number => str)
    fn current_line(&self, offset: usize) -> Result<Line, Error>;

    /// Get the last `count` number of lines (inclusive)
    fn last_lines(&self, offset: usize, count: usize) -> Result<Vec<Line>, Error>;

    /// Get the next `count` number of lines (inclusive)
    fn next_lines(&self, offset: usize, count: usize) -> Result<Vec<Line>, Error>;
}

//TODO: not yet implemented in solc_api
pub trait Ast {
    type Err;
    /// Get a contract by it's byte offset in the source file
    fn contract_by_offset(&self, offset: u32) -> Result<String, Self::Err>;
}

