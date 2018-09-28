mod source_map;
mod ast;
pub mod err;

use std::{
    path::PathBuf,
    io::Read,
    sync::Arc,
};
use itertools::*;
use web3::Transport;
use log::*;

use solc_api::{
    CompiledSource, SolcApiBuilder, Contract as CompiledContract,
    types::FoundationVersion
};
use self::{err::SolidityError, source_map::SoliditySourceMap};
use super::{SourceMap, Language, ContractFile, Contract, err::LanguageError };

/// A struct for Solidity Source Mapping
#[derive(Debug, Clone, PartialEq)]
pub struct Solidity;
/*
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
    */

impl Language for Solidity {

    fn compile<T>(&self, path: PathBuf, eth: &web3::api::Eth<T>)
        -> Result<Vec<ContractFile<T>>, LanguageError>
        where
            T: Transport
    {
        let mut source = String::new();
        info!("Read {} bytes from Source File", std::fs::File::open(path.as_path())?.read_to_string(&mut source)?);

        let parent = path.parent().ok_or(SolidityError::ParentNotFound)?;
        let compiled_source = SolcApiBuilder::default()
            .source_file(path)
            .evm_version(FoundationVersion::Byzantium)
            .compile();
        let contracts = compiled_source
            .sources()
            .map(|file, compiled_file| {
                let contracts = compiled_source
                    .contracts_by(|c| c.file_name == file)
                    .map(|c| {
                        // if
                        let import_path = parent.push(PathBuf::from(file.as_str()));
                        let mut src = String::new();
                        info!("Read {} bytes from source file: {}", std::fs::File::open(import_path.as_path())?.read_to_string(&mut src)?, file);
                        Contract::new(c.name, eth.clone(), SoliditySourceMap::new(src.as_str()), c.abi, c.evm.bytecode.object)
                    })
                    .collect::<Result<Vec<CompiledContract, LanguageError>>>();
                ContractFile::new(&source, source.id, )
            })
            .collect::<Vec<ContractFile<Solidity>>>();
        Ok(contracts)
    }
}
/*
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

*/
