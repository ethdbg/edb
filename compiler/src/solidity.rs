mod source_map;
mod standard_json;
mod ast;
use std::{
    path::PathBuf,
    io::Read,
};
use log::*;
use self::{
    standard_json::{CompiledSource, StandardJsonBuilder},
    source_map::SoliditySourceMap,
};
use super::{
    types::FoundationVersion,
    SourceMap
};

/// A struct for Solidity Source Mapping
pub struct Solidity {
    /// Source code as a string. No transformations done on it.
    source: String,
    compiled_source: CompiledSource,
    maps: Vec<Mapping>
    // ast: AST,
}

struct Mapping {
    file: String,
    contract_name: String,
    index: usize,
    map: SoliditySourceMap
}

/// Solidity Compiler Interface
// need: ABI, AST, SourceMap, bincode-runtime
impl Solidity {
    pub fn new(path: PathBuf) -> Self {
        let mut source = String::new();
        info!("Read {} bytes from Source File", std::fs::File::open(path.as_path()).unwrap().read_to_string(&mut source).unwrap());
        let compiled_source = StandardJsonBuilder::default()
            .source_file(path)
            .evm_version(FoundationVersion::Byzantium)
            .compile();
        let maps = compiled_source.contracts
            .iter()
            .enumerate()
            .flat_map(|(i, (k, v))| {
                v
                    .iter()
                    .map(|(inner_k, inner_v)| {
                        Mapping {
                            file: k.clone(),
                            contract_name: inner_k.clone(),
                            index: i,
                            map: SoliditySourceMap::new(&inner_v.evm.deployed_bytecode.as_ref().expect("Bytecode doesn't exist!").source_map)
                        }
                    }).collect::<Vec<Mapping>>()
            })
            .collect::<Vec<Mapping>>();

        Solidity { source, compiled_source, maps }
    }
}

// Decompress Source Mappings
// Store in data structure Line No -> SrcMapping
impl SourceMap for Solidity {
    fn pc_from_lineno(&self, lineno: usize) -> usize {
        unimplemented!();
    }
}


#[cfg(test)]
mod test {
    use speculate::speculate;
    use log::*;
    use super::*;
    speculate! {
        before {
            pretty_env_logger::try_init();
        }
        describe "solidity" {
            it "should compile source" {
                Solidity::new(PathBuf::from("./../tests/contracts/solidity/voting/voting.sol"));
            }
        }
    }
}
