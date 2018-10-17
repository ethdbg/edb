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
// TODO: Consider the tradeoffs of adding lifetimes to CodeFile and traits.
//  - would slightly complicate public facing API, and force users to use lifetimes themselves if
//  they want to put traits/structs/whatever in a composition
//  - but would allow storing of direct references
//  - work better with Implementations which themselves put lifetimes on structs
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
use web3::{Transport, types::{Address}};
use failure::Error;


/// The Source File of a specific language
pub trait Language {
    /// Compiles Source Code File into a Vector of Contract Files
    fn compile<T>(&self, path: PathBuf, client: &web3::api::Eth<T>, addresses: &[Address])
        -> Result<(Vec<Rc<ContractFile>>, Vec<Contract<T>>), Error> where T: Transport;
}

/// Represents a Line - Line number and String (0-indexed)
pub type Line = (usize, String);
/// A Line Number
pub type LineNo = usize;
/// Offset into the bytecode
pub type OpcodeOffset = usize;
/// Offset into the source file
pub type CharOffset = usize;
pub type SourceRange = (usize, usize);

//TODO: Can merge some of these functions by passing in an enum
/// Represents a Source Map
pub trait SourceMap {

    /// Check if a unique opcode mapping exists
    /// Generally used for setting breakpoints
    fn unique_exists(&self, lineno: LineNo) -> bool;

    /// Get a unique linenumber mapped to an opcode position
    /// This is usually the instruction in the sourcemap with the shortest length, that matches the
    /// linenumber provided. Usually used for run_until().
    /// Generally this ignores function declarations, while() loops, and if() statements used
    /// for breakpoint handling
    fn unique_opcode_pos(&self, lineno: LineNo) -> Result<OpcodeOffset, Error>;

    /// Get the instruction offset from a line number in the Source Code.
    /// This is the first occurrence of an opcode relative to `from` offset that matches the
    /// linenumber provided. Usually used for step()
    fn opcode_pos_from_lineno(&self, lineno: LineNo, from: OpcodeOffset) -> Result<OpcodeOffset, Error>;

    /// Get the character position in a file from a line number (Ignores leading whitespace)
    fn char_pos_from_lineno(&self, lineno: LineNo) -> Result<CharOffset, Error>;

    /// Get the LineNumber that corresponds with a character offset
    fn lineno_from_char_pos(&self, offset: CharOffset) -> Result<LineNo, Error>;

    /// Get the linenumber that corresponds to an opcode position
    fn lineno_from_opcode_pos(&self, offset: OpcodeOffset) -> Result<LineNo, Error>;

    /// Get a line mapping (line number => str) from opcode position/offset
    fn current_line(&self, offset: OpcodeOffset) -> Result<Line, Error>;

    /// Get the last `count` number of lines (inclusive) from opcode position/offset
    fn last_lines(&self, offset: OpcodeOffset, count: usize) -> Result<Vec<Line>, Error>;

    /// Get the next `count` number of lines (inclusive) from opcode position/offset
    fn next_lines(&self, offset: OpcodeOffset, count: usize) -> Result<Vec<Line>, Error>;
}


/// loosely and generally represents a Node in the Ast attached to a particular language item
#[derive(Debug, Clone, PartialEq)]
pub struct AstItem {
    pub variant: AstType,
    pub name: String,
    pub location: SourceRange,
}

/// Type of node being represented
#[derive(Debug, Clone, PartialEq)]
pub enum AstType {
    /// Contract Declaration
    Contract,
    /// Variable/const declaration
    VarDeclaration,
}

pub trait AbstractFunction {
    /// Name of the function
    fn name(&self) -> String;
    /// Parameters of function
    fn params(&self) -> ethabi::Param;
    /// Function Returns
    fn returns(&self) -> ethabi::Param;
    /// Any mutations to state that occur within the function
    fn mutations(&self) -> Box<Iterator<Item=Mutation>>;
    fn location(&self) -> SourceRange;
}

/// Enum representing the mutations to state that may occur within a function body
pub enum Mutation {
    LocalMutation(Variable, Variable),
    InstanceMutation(Variable, Variable),
}

/// General variable type
pub struct Variable {
    name: String,
    var_type: VariableType
}

/// Types that may be used within source code
pub enum VariableType {
    Address,
    Bytes,
    Mapping,
    Int(usize),
    Uint(usize),
    Bool(bool),
    String(String),
}

pub trait Ast {
    /// get a variable declaration
    fn variable(&self, name: &str) -> Result<AstItem, Error>;
    /// Get a contract declaration
    fn contract(&self, name: &str) -> Result<AstItem, Error>;
    /// Access a Function via a Closure
    fn function(&self, name: &str, fun: &mut FnMut(Result<&AbstractFunction, Error>)) -> Result<(), Error>;
    /// Find a contract by it's byte offset in the source file
    fn find_contract(&self, offset: CharOffset) -> Option<AstItem>;
    /// Find a function name by a character offset in source code. Resulting function information is accessed
    /// via a closure
    // the reason for a closure here is to avoid having to redundantly re-describe all the types
    // which may already be described within an Ast implementation (Ex: Lunarity). This preserves
    // more information about the function too.
    fn find_function(&self, offset: CharOffset, fun: &mut FnMut(Option<&AbstractFunction>));
}

