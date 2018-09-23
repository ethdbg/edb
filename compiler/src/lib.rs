#![recursion_limit="128"]
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
mod source_map;
pub mod code_file;
// pub mod solidity;
// pub mod vyper;
use self::err::LanguageError;

use std::{
    path::PathBuf,
    slice::Iter,
};
use web3::{
    contract::{Contract as Web3Contract},
    Transport,
};
use delegate::*;

/// The Source File of a specific language
pub trait Language {
    type Err;
    // Language Functions
    /// get the ABI of a contract
    fn abi(&self, contract_name: &str) -> ethabi::Contract;
    /// raw source file
    fn source(&self) -> &str;
    /// file name
    fn file_name(&self) -> &str;
    /// file path
    fn file_path(&self) -> PathBuf;

    /// Returns an Iterator over all contracts
    fn contracts<T>(&self) -> Iter<Contract<T>> where T: Transport;

    /// Deploys all contracts in source
    fn deploy<T>(&self, client: web3::Web3<T>) -> Result<Contract<T>, Self::Err> where T: Transport;
}

/// Represents a Line - Line number and String (0-indexed)
pub type Line = (u32, String);

// 32bit integers, which is enough to support a 4GB file.
// if someone is commiting a contract to Ethereum larger than 4GB, call me I want to know about it
/// Represents a Source Map
pub trait SourceMap {
    type Err;
    /// Get the instruction offset from a line number in the Source Code
    /// Optional File - if not specified, takes first file in index
    fn position_from_lineno(&self, file: &FileIdentifier, lineno: u32) -> u32;

    /// The reverse of `position_from_lineno`
    fn lineno_from_position(&self, file: &FileIdentifier, offset: u32) -> u32;

    /// Get a line mapping (line number => str)
    fn current_line(&self, offset: u32) -> Result<Line, Self::Err>;

    /// Get the last `count` number of lines (inclusive)
    fn last_lines(&self, offset: u32, count: u32) -> Result<Vec<Line>, Self::Err>;

    /// Get the next `count` number of lines (inclusive)
    fn next_lines(&self, offset: u32, count: u32) -> Result<Vec<Line>, Self::Err>;
}

// this may be totally unnecessary
/// Identifier for where code resides.
pub enum FileIdentifier {
    /// Identified by file name
    File(String),
    /// Identified by Contract Name
    Contract(String),
    /// Identified by index in compiled code (IE: Standard JSON)
    Index(usize),
}
/*
trait CrawlMapping {
    fn try_from_file(&self, file: &FileIdentifier) -> Option<&Mapping>;
}

impl CrawlMapping for Box<dyn SourceMap<Err=LanguageError>> {
    fn try_from_file(&self, file: &FileIdentifier) -> Option<&Mapping> {
        match file {
            // TODO: Should return a vector, in case multiple contracts are contained within one file
            FileIdentifier::File(file_name) => self.get_mapping(|m| m.file == file_name),
            FileIdentifier::Contract(c_name) => self.get_mapping(|m| m.contract_name = c_name),
            FileIdentifier::Index(idx) => self.get_mapping(|m| m.index = idx)
        }
    }
}
 */

//TODO: not yet implemented in solc_api
pub struct Ast;

// TODO
// probably will have to be made generic,
// but requirements for Vyper/LLL are not yet clear, so this mostly
// caters to Solidity
pub struct ContractFile<T> where T: Transport {
    /// Identifier for source file (used in Source Maps)
    id: usize,
    /// All the contracts contained in the souce
    contracts: Vec<Contract<T>>,
    /// path to source file
    file_path: PathBuf,
    /// name of source file
    file_name: String,
    /// Abstract Syntax Tree of Source
    ast: Ast,
}

/// Contract
pub struct Contract<T> where T: Transport {
    name: String,
    abi: ethabi::Contract,
    deployable: web3::contract::Contract<T>,
    bytecode: Vec<u8>,
    runtime_bytecode: Vec<u8>,
    source_map: Box<dyn SourceMap<Err=LanguageError>>
}

// contract interface for the debugger. should be the same across all languages
impl<T> Contract<T> where T: Transport {
    delegate! {
        target self.abi {
            // loads contract abi from JSON
            // pub fn load<T: std::io::Read>(reader: T) -> Result<Self, ethabi::Error>;

            /// Creates abi constructor call builder
            pub fn constructor(&self) -> Option<&ethabi::Constructor>;
            /// Creates abi function call builder
            pub fn function(&self, name: &str) -> Result<&ethabi::Function, ethabi::Error>;
            /// Creates abi event decoder
            pub fn event(&self, name: &str) -> Result<&ethabi::Event, ethabi::Error>;
            /// Iterate over all functions of the contract in arbitrary order
            pub fn functions(&self) -> ethabi::Functions;
            /// iterate over all events of the contract abi in arbitrary order
            pub fn events(&self) -> ethabi::Events;
            /// Returns true if contract abi has fallback
            pub fn fallback(&self) -> bool;
        }
    }

    /// Deploys a specific contract in source
    fn deploy(&self, client: web3::api::Eth<T>) -> Result<(), LanguageError> {
        unimplemented!();
    }

    fn address(&self) -> web3::types::Address {
        unimplemented!();
    }

    /// get the source map of this contract
    fn source_map(&self) -> Box<dyn SourceMap<Err=LanguageError>> {
        self.source_map
    }
}
