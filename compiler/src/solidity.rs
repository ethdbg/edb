mod source_map;
mod ast;
pub mod err;

use std::{
    path::PathBuf,
    io::Read,
    iter::FromIterator,
    rc::Rc,
};
use web3::{Transport, types::Address};
use failure::Error;
use solc_api::{ SolcApiBuilder, types::FoundationVersion };
use log::*;
use self::{err::SolidityError, source_map::SoliditySourceMap, ast::SolidityAst};
use super::{Language, contract::{ContractFile, Contract} };

/// A struct for Solidity Source Mapping
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Solidity;

impl Language for Solidity {

    fn compile<T>(&self, path: PathBuf, eth: &web3::api::Eth<T>, address: &Address)
        -> Result<(Vec<Rc<ContractFile>>, Vec<Contract<T>>), Error>
        where
            T: Transport
    {
        let mut source = String::new();
        let file = std::fs::File::open(path.as_path())?.read_to_string(&mut source)?;
        info!("Read {} bytes from Source File", file);

        let parent = path.parent().ok_or(SolidityError::ParentNotFound)?.to_path_buf();
        let compiled_source = SolcApiBuilder::default()
            .source_file(path)
            .evm_version(FoundationVersion::Byzantium)
            .compile();

        let mut contracts = Vec::new();
        let files = compiled_source
            .sources()
            .map(|(file, compiled_file)| {
                let mut import_path = parent.clone();
                import_path.push(PathBuf::from(file.as_str()));
                let mut src = String::new();
                let file_buf = std::fs::File::open(import_path.as_path())?.read_to_string(&mut src)?;
                info!("Read {} bytes from source file: {}", file_buf, file);

                let ast = SolidityAst::new(&src)?;
                let cfile = Rc::new(ContractFile::new(src, compiled_file.id, Box::new(ast), import_path)?);
                contracts.extend(compiled_source
                    .contracts_by(|c| &c.file_name == file)
                    .map(|c| {
                        let deployed_code = c.evm.deployed_bytecode.as_ref().expect("Should never be missing field bytecode; qed").clone();
                        Contract::new(cfile.clone(),
                                      c.name.clone(),
                                      eth.clone(),
                                      Box::new(SoliditySourceMap::new(cfile.clone().source(), deployed_code.source_map)),
                                      c.abi.clone(),
                                      address,
                                      deployed_code.object
                                      ).map_err(|e| e.into())
                    }));
                Ok(cfile)
            })
            .collect::<Result<Vec<Rc<ContractFile>>, Error>>()?;
        let contracts = Result::<Vec<Contract<T>>, Error>::from_iter(contracts)?;
        Ok((files, contracts))
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
        Solidity::compile(&Solidity, path, &client.eth(), edbtest::eth_contract_addrs().as_slice()).unwrap();
    }
}
