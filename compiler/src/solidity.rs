mod source_map;
use std::path::PathBuf;
use serde_derive::*;
use super::types::*;

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


    }
}

#[derive(Serialize, Debug, Clone)]
struct SolcBuilder {
    /// specify the path of the source code
    source: PathBuf,
    /// a array of File Paths to source code
    urls: Vec<PathBuf>,
    /// EvmVersion to use
    version: Option<String>,
    /// [FLAG] whether to optimize output
    optimize: Option<bool>,
}

impl SolcBuilder {
    // fs::canonicalize
    fn source_file(&mut self, val: PathBuf) -> &mut Self {
        let new = self;
        new.source = val;
        new
    }

    fn evm_version(&mut self, ver: FoundationVersion) -> &mut Self {
        let new = self;
        new.version = Some(ver.into());
        new
    }

    fn optimize(&mut self) -> &mut self {
        let new = self;
        new.optimize = Some(true);
        new
    }

    fn build(&self) -> String { // returns standard JSON input for solidity compiler

    }
}
