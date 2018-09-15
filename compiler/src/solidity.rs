mod source_map;
mod standard_json;
mod ast;
use std::path::PathBuf;
use self::ast::AST;

/// A struct for Solidity Source Mapping
pub struct Solidity {
    /// Source code as a string. No transformations done on it.
    source: String,
    ast: AST,
}

/// Solidity Compiler Interface
// need: ABI, AST, SourceMap, bincode-runtime
impl Solidity {
    pub fn new(path: PathBuf) -> Self {
        unimplemented!();
    }
}
