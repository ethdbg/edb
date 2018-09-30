mod source_map;
mod ast;
pub mod err;

use std::{
    path::PathBuf,
    io::Read,
};
use web3::Transport;
use log::*;
use failure::Error;
use solc_api::{
    SolcApiBuilder, Contract as CompiledContract,
    types::FoundationVersion
};
use self::{err::SolidityError, source_map::SoliditySourceMap, ast::SolidityAst};
use super::{Language, ContractFile, Contract, err::LanguageError };

/// A struct for Solidity Source Mapping
#[derive(Debug, Clone, PartialEq)]
pub struct Solidity;

impl Language for Solidity {

    fn compile<T>(&self, path: PathBuf, eth: &web3::api::Eth<T>)
        -> Result<Vec<ContractFile<T>>, Error>
        where
            T: Transport
    {
        let mut source = String::new();
        info!("Read {} bytes from Source File", std::fs::File::open(path.as_path())?.read_to_string(&mut source)?);

        let parent = path.parent().ok_or(SolidityError::ParentNotFound)?.to_path_buf();
        let compiled_source = SolcApiBuilder::default()
            .source_file(path)
            .evm_version(FoundationVersion::Byzantium)
            .compile();

        let contracts = compiled_source
            .sources()
            .map(|(file, compiled_file)| {
                let mut import_path = parent.clone();
                import_path.push(PathBuf::from(file.as_str()));
                let mut src = String::new();
                info!("Read {} bytes from source file: {}", std::fs::File::open(import_path.as_path())?.read_to_string(&mut src)?, file);
                let contracts = compiled_source
                    .contracts_by(|c| &c.file_name == file)
                    .map(|c| {
                        let deployed_code = c.evm.deployed_bytecode.as_ref().expect("Should never be missing field bytecode; qed").clone();
                        Contract::new(c.name.as_str(),
                                      eth.clone(),
                                      Box::new(SoliditySourceMap::new(&src.as_str(), deployed_code.source_map)),
                                      c.abi.clone(),
                                      deployed_code.object
                                      ).map_err(|e| e.into())
                    })
                    .collect::<Result<Vec<Contract<T>>, Error>>()?;
                ContractFile::new(&src, compiled_file.id, contracts, Box::new(SolidityAst::new(&src)?), import_path)
            })
            .collect::<Result<Vec<ContractFile<T>>, Error>>()?;
        Ok(contracts)
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use log::*;
    use super::*;
    use edb_test_helpers as edbtest;

    #[test]
    fn compile_solidity() {
        pretty_env_logger::try_init();
        let mock = edbtest::MockWeb3Transport::default();
        let client = web3::Web3::new(mock);
        let path = edbtest::contract_path(edbtest::Contract::Voting);
        Solidity::compile(&Solidity, path, &client.eth()).unwrap();
    }
}
