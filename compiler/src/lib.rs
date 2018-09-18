#![feature(non_modrs_mods)]
//! Traits 'Compiler' modules must implement
//!
//!
pub mod solidity;
mod vyper;
mod err;
mod types;

#[macro_use] extern crate serde_derive;
extern crate failure;
extern crate log;
extern crate pretty_env_logger;
extern crate solc;
extern crate ethereum_types;
extern crate serde_json;
extern crate serde;
extern crate ethabi;
extern crate url;
extern crate itertools;
extern crate codespan;

#[cfg(test)]
#[macro_use] extern crate speculate;


/*
 * TBA trait for walking, and getting information from AST
 */
pub trait SourceMap {
    /// Get the instruction offset from a line number in the Source Code
    /// Optional File - if not specified, takes first file in index
    fn position_from_lineno(&self, file: &FileIdentifier, lineno: usize) -> usize;
    /// Get the source code
    fn source(&self) -> &str;
    /// get the ABI of a contract
    fn abi(&self, contract_name: &str) -> ethabi::Contract;
}

/// Functions
pub trait Compiler {
    /* compile(path: PathBuf) -> Box<Ast>; */
}


/// Identifier for where code resides.
pub enum FileIdentifier {
    /// Identified by file name
    File(String),
    /// Identified by Contract Name
    Contract(String),
    /// Identified by index in compiled code (IE: Standard JSON)
    Index(usize),
}
