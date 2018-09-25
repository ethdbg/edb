#![recursion_limit="128"]
#![feature(test)]
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
mod map;
pub mod code_file;
// pub mod solidity;
// pub mod vyper;
use self::err::{LanguageError, NotFoundError};
use serde_derive::*;
use std::{
    path::PathBuf,
    slice::Iter,
};
use web3::{
    types::{Address, BlockNumber},
    Transport,
};
use futures::future::Future;
use delegate::*;
extern crate test;

/// The Source File of a specific language
pub trait Language<'a> {
    // Language Functions
    /// Returns an Iterator over all contracts in a source file
    fn contracts<T>(&self) -> Iter<Contract<T>> where T: Transport;
    fn compile<T>(&self, path: PathBuf) -> Result<Vec<ContractFile<'a, T>>, LanguageError> where T: Transport;
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

//TODO: not yet implemented in solc_api
pub trait Ast {
    type Err;
    fn contract_by_offset(&self, offset: u32) -> Result<String, Self::Err>;
}

pub struct ContractFile<'a, T> where T: Transport {
    /// Identifier for source file (used in Source Maps)
    id: usize,
    /// All the contracts contained in the souce
    contracts: &'a [Contract<T>],
    /// path to source file
    file_path: &'a str,
    /// name of source file
    file_name: &'a str,
    /// General source map for offsets--line number
    map: self::map::Map<'a>,
    // Abstract Syntax Tree of Source
    // ast: Ast<Err=LanguageError>,
}

/// Contract
pub struct Contract<T> where T: Transport {
    name: String,
    abi: ethabi::Contract,
    deployed: web3::contract::Contract<T>,
    bytecode: Vec<u8>,
    runtime_bytecode: Vec<u8>,
    source_map: Box<dyn SourceMap<Err=LanguageError>>,
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

    pub fn new(eth: web3::api::Eth<T>,
               name: &str,
               map: impl SourceMap<Err=LanguageError>,
               abi: ethabi::Contract) -> Result<Self, LanguageError>
    {
        unimplemented!()
    }

    // TODO: Make parallel/async
    /// Find a contract from it's bytecode and a local ethereum node
    fn find_deployed_contract(needle: &[u8], eth: web3::api::Eth<T>) -> Result<Address, LanguageError> {

        let accounts = eth.accounts().wait()?;

        for a in accounts.iter() {
            let code = eth.code(*a, Some(BlockNumber::Latest)).wait()?;
            if needle == code.0.as_slice() {
                return Ok(*a);
            }
        }
        return Err(LanguageError::NotFound(NotFoundError::Contract))
    }

    /// Returns address on testnet that the contract is deployed at
    pub fn address(&self) -> Address {
        self.deployed.address()
    }

    /// get the source map of this contract
    pub fn source_map(&self) -> &Box<dyn SourceMap<Err=LanguageError>> {
        &self.source_map
    }
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
