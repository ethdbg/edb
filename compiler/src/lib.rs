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
pub mod map;
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
pub trait Language {
    // Language Functions
    /// Returns an Iterator over all contracts in a source file (including imported contracts)
    fn contracts<T>(&self) -> Iter<Contract<T>> where T: Transport;
    fn compile<T>(&self, path: PathBuf) -> Result<Vec<ContractFile<T>>, LanguageError> where T: Transport;
}

/// Represents a Line - Line number and String (0-indexed)
pub type Line = (u32, String);

/// Represents a Source Map
pub trait SourceMap {
    type Err;
    /// Get the instruction offset from a line number in the Source Code
    /// Optional File - if not specified, takes first file in index
    fn position_from_lineno(&self, lineno: usize) -> usize;

    /// The reverse of `position_from_lineno`
    fn lineno_from_position(&self, offset: usize) -> usize;

    /// Get a line mapping (line number => str)
    fn current_line(&self, offset: usize) -> Result<Line, Self::Err>;

    /// Get the last `count` number of lines (inclusive)
    fn last_lines(&self, offset: usize, count: usize) -> Result<Vec<Line>, Self::Err>;

    /// Get the next `count` number of lines (inclusive)
    fn next_lines(&self, offset: usize, count: usize) -> Result<Vec<Line>, Self::Err>;
}

//TODO: not yet implemented in solc_api
pub trait Ast {
    type Err;
    /// Get a contract by it's byte offset in the source file
    fn contract_by_offset(&self, offset: u32) -> Result<String, Self::Err>;
}

pub struct ContractFile<T> where T: Transport {
    /// Identifier for source file (used in Source Maps)
    id: usize,
    /// All the contracts contained in the souce
    contracts: Vec<Contract<T>>,
    /// path to source file
    file_path: PathBuf,
    /// name of source file
    file_name: String,
    /// General source map for offsets--line number
    map: self::map::Map,
    // Abstract Syntax Tree of Source
    ast: Box<dyn Ast<Err=LanguageError>>,
}

impl<T> ContractFile<T> where T: Transport {
    pub fn new(source: &str, id: usize, contracts: Vec<Contract<T>>, ast: Box<dyn Ast<Err=LanguageError>>, file_path: PathBuf)
               -> Result<Self, LanguageError>
    {
        let file_name = file_path
            .file_name()
            .ok_or(LanguageError::FileNotFound)?
            .to_str()
            .ok_or(LanguageError::InvalidPath)?
            .to_string();

        Ok(Self {
            map: self::map::Map::new(source),
            file_name, file_path, id, contracts, ast
        })
    }
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

    pub fn new(name: &str,
               eth: web3::api::Eth<T>,
               map: Box<dyn SourceMap<Err=LanguageError>>,
               abi: ethabi::Contract,
               bytecode: Vec<u8>,
               runtime_bytecode: Vec<u8>,
    ) -> Result<Self, LanguageError>
    {
        let contract = web3::contract::Contract::new(
            eth.clone(),
            Self::find_deployed_contract(runtime_bytecode.as_slice(), &eth)?,
            abi.clone()
        );
        Ok(Self {
            name: name.to_string(),
            abi,
            deployed: contract,
            bytecode,
            runtime_bytecode,
            source_map: map
        })
    }

    // TODO: Make parallel/async
    /// Find a contract from it's bytecode and a local ethereum node
    fn find_deployed_contract(needle: &[u8], eth: &web3::api::Eth<T>)
                              -> Result<Address, LanguageError>
    {

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
