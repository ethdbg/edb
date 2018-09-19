//! Traits 'Compiler' modules must implement
//!
//!


pub mod solidity;
mod vyper;
mod err;
mod types;


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
