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
mod contract;
mod source_map;
pub mod code_file;
pub trait Language {
    // Language Functions
    /// get the ABI of a contract
    fn abi(&self, contract_name: &str) -> ethabi::Contract;
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

    /// Get the source code
    fn source(&self) -> &str;

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
