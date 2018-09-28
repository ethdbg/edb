mod source_map;
mod ast;
mod err;

use std::{
    path::PathBuf,
    io::Read,
    sync::Arc,
};
use log::*;

use solc_api::{CompiledSource, SolcApiBuilder, Contract, types::input::FoundationVersion};
use super::map::Map;
use self::{
    err::{SolidityError},
};

use super::{SourceMap, Language, ContractFile, Contract};

/// A struct for Solidity Source Mapping
pub struct Solidity {
    /// Source code as a string. No transformations done on it.
    source: String,
    /// Compiled Source (via standard-json api)
    compiled_source: CompiledSource,
}

pub struct SoliditySourceMap { }


/// Solidity Compiler Interface
impl Solidity {
    pub fn new(path: PathBuf) -> Result<Self, SolidityError> {
        let mut source = String::new();
        info!("Read {} bytes from Source File", std::fs::File::open(path.as_path())?.read_to_string(&mut source)?);

        let compiled_source = SolcApiBuilder::default()
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


                    }).collect::<Vec<Mapping>>()
            })
            .collect::<Vec<Mapping>>();

        Ok(Solidity { code_map, file_map, source, compiled_source, maps })
    }

    // find the mapping with the shortest length from the byte offset
    fn shortest_len<'a>(&self, lineno: u32, mapping: &'a Mapping) -> Option<&'a Instruction> {
        mapping.map.instructions
            .iter()
            .fold(None, |min, x| {

                match min {
                    None => if self.file_map.find_line(ByteIndex(x.start as u32)).expect("failed to find line") == LineIndex(lineno) { Some(x) } else { None },
                    Some(y) => {
                        println!("ACC: {}, X: {}", y.start, x.start);
                        info!("Found: Acc: {}, acclength: {}, x: {}, x_length: {}", y.start, y.length, x.start, x.length);
                        if self.file_map.find_line(ByteIndex(x.start as u32)).expect("failed to find line") == LineIndex(lineno) && x.length < y.length {
                            Some(x)
                        } else {
                            Some(y)
                        }
                    }
                }
            })
    }
}

#[cfg(test)]
mod test {
    use speculate::speculate;
    #[allow(unused_imports)]
    use log::*;
    use super::*;

    speculate! {
        before {
            // #[allow(unused_must_use)] {
                // pretty_env_logger::try_init();
            // }
            let _path = PathBuf::from("./../tests/contracts/solidity/voting/voting.sol");
        }
        describe "solidity" {
            it "should compile source" {
                Solidity::new(_path);
            }
            it "should get a mapping" {
                let sol = Solidity::new(_path);
                // info!("Map:{:?}", sol.maps);
                let pos: usize = sol.position_from_lineno(&FileIdentifier::File("voting.sol".into()), 117);
                // assert_eq!(pos, 1427);
                info!("Position: {}", pos)
            }
        }
    }
}
