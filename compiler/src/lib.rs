//! Traits 'Compiler' modules must implement
//!
//!
mod vyper;
mod solidity;
mod err;
mod types;

/*
 * TBA trait for walking, and getting information from AST
 */
pub trait SourceMap {
    /// Get a PC from a line number in the Source Code
    fn pc_from_lineno(&self, lineno: usize) -> usize;
}

/// Functions
pub trait Compiler {
    /* compile(path: PathBuf) -> Box<Ast>; */
}
