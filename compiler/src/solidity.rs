mod source_map;
mod standard_json;
mod ast;
use std::{
    path::PathBuf,
    io::Read,
};
use log::*;
use self::ast::AST;

/// A struct for Solidity Source Mapping
pub struct Solidity {
    /// Source code as a string. No transformations done on it.
    source: String,
    // ast: AST,
}

/// Solidity Compiler Interface
// need: ABI, AST, SourceMap, bincode-runtime
impl Solidity {
    pub fn new(path: PathBuf) -> Self {
        let mut source = String::new();
        info!("Read {} bytes from Source File", std::fs::File::open(path.as_path()).unwrap().read_to_string(&mut source).unwrap());
        Solidity {
            source,
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn compiles_source() {
        Solidity::new(PathBuf::from("./../tests/contracts/solidity/voting/voting.sol"));
    }
}
