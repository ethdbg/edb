//! Traits 'compiler' modules must implement
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
pub mod solidity;
// mod vyper;
mod err;
mod types;

pub trait Language {
    // Language Functions
}

pub trait Contract {
    // Contract Interface Functions
}

/*
 * TBA trait for walking, and getting information from AST
 */
pub trait SourceMap { // maybe rename to `BytecodeSourceMap`
    /// Get the instruction offset from a line number in the Source Code
    /// Optional File - if not specified, takes first file in index
    fn position_from_lineno(&self, file: &FileIdentifier, lineno: u32) -> usize;
    /// Get the source code
    fn source(&self) -> &str;
    /// get the ABI of a contract
    fn abi(&self, contract_name: &str) -> ethabi::Contract;
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
