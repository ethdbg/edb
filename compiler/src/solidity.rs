mod source_map;
mod ast;
mod err;

use std::{
    path::PathBuf,
    io::Read,
    sync::Arc,
};
use log::*;
use codespan::{
    CodeMap, FileMap, ByteIndex, LineIndex
};
use solc_api::{CompiledSource, SolcApiBuilder, Contract, types::input::FoundationVersion};
use self::{
    source_map::{SoliditySourceMap, Instruction},
    err::{SolidityError, SourceMapVariant},
};
use super::{SourceMap, FileIdentifier};

/// A struct for Solidity Source Mapping
pub struct Solidity {
    /// Source code as a string. No transformations done on it.
    source: String,
    /// Compiled Source (via standard-json api)
    compiled_source: CompiledSource,
    /// Source Mappings
    maps: Vec<Mapping>,
    /// Different operations for seeking through file. Built on top of B-Tree's
    code_map: CodeMap,
    file_map: Arc<FileMap>, // temporary, for demo
    // ast: AST, (unimplemented)
}

#[derive(Debug)]
struct Mapping {
    /// File mapping is contained in
    file: String,
    ///
    contract_name: String,
    index: usize,
    map: SoliditySourceMap,
}

/// Solidity Compiler Interface
// need: ABI, AST, SourceMap, bincode-runtime
impl Solidity {
    pub fn new(path: PathBuf) -> Result<Self, SolidityError> {
        let mut source = String::new();
        info!("Read {} bytes from Source File", std::fs::File::open(path.as_path())?.read_to_string(&mut source)?);

        let mut code_map = CodeMap::new();
        let file_map = code_map.add_filemap_from_disk(path.as_path())?;

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
                        let src_map = SoliditySourceMap::new(&inner_v.evm.deployed_bytecode.as_ref()
                                                             .expect("Bytecode does not exist").source_map)
                            .expect("Fatal Error: Could not build Source Map. Shutting Down...");
                        Mapping {
                            file: k.clone(),
                            contract_name: inner_k.clone(),
                            index: i,
                            map: src_map,
                        }
                    }).collect::<Vec<Mapping>>()
            })
            .collect::<Vec<Mapping>>();

        Ok(Solidity { code_map, file_map, source, compiled_source, maps })
    }

    /// get a mapping with the predicate 'f'
    fn get_mapping<F>(&self, fun: F) -> Option<&Mapping>
    where
        F: Fn(&[Mapping]) -> bool
    {

        self.maps
            .iter()
            .find(fun)
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

    fn contract_by_name(&self, name: &str) -> Option<Contract> {
        // TODO: this iteration is really bad. fix it.
        self.compiled_source.contracts
            .iter()
            .map(|(_, v)| {
                v
                    .iter()
                    .find(|(k2, _)| k2.as_str() == name)
            })
            .collect::<Vec<Option<(&String, &Contract)>>>()
            .iter()
            .filter_map(|o| o.clone())
            .find(|(s, c)| {
                s.as_str() == name
            })
            .and_then(|(s, c)| Some(c.clone()))
            .map(|c| c)
    }
    /*
    pub fn get_current_line(&self, offset: u32) -> Result<(u32, String), SolidityError> {
        let line_num = self.file_map.find_line(ByteIndex(offset))?;
        let lines = self.source
            .lines()
            .map(|s| {
                s.to_string()
            })
            .collect::<Vec<String>>();
            info!("Line num: {}", line_num.0);
        let line_str = lines.get(line_num.0 as usize);
        Ok((line_num.0, line_str.expect("No line str").clone()))
    }
    */
}

// Decompress Source Mappings
// Store in data structure Line No -> SrcMapping
impl SourceMap for Solidity {
    fn position_from_lineno(&self, file: &FileIdentifier, lineno: u32) -> usize {
        // TODO: Maybe a impl on the enum, or a trait implemented on enum?
        match file {
            FileIdentifier::File(name) => {
                self.shortest_len(lineno, self.get_mapping(|m| m.file == name).expect("Could not get mapping from file")).expect("Could not get shortest length").position
            },
            FileIdentifier::Contract(name) => {
                self.shortest_len(lineno, self.get(name).unwrap()).unwrap().position
            },
            FileIdentifier::Index(idx) => {
                self.shortest_len(lineno, self.mapping_by_index(*idx).unwrap()).unwrap().position
            }
        }
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn abi(&self, contract_name: &str) -> ethabi::Contract {
        self.contract_by_name(contract_name).unwrap().abi
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
