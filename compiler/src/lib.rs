//! Traits 'Compiler' modules must implement
//!
//!
mod vyper;
mod solidity;

/*
 * TBA trait for walking, and getting information from AST
pub trait Ast {

}
*/

/// Functions 
pub trait Compiler {
    /* compile(path: PathBuf) -> Box<Ast>; */
    /// Get a PC from a line number in the Source Code
    fn pc_from_lineno(lineno: usize) -> usize;
    /// Other Functions ... (TBA)
}
