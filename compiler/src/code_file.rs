//! Codefile represents one source code file and all of the files it imports
use super::{Language, Line, Offset, LineNo, Contract, ContractFile, err::{LanguageError, NotFoundError}};
use failure::Error;
use web3::Transport;
use std::path::{PathBuf};

// every CodeFile is associated with a language
pub struct CodeFile<L: Language, T: Transport> {
    language: L,
    client: web3::Web3<T>,
    name: String,
    files: Vec<ContractFile<T>>,
}

// TODO: Assumes all contracts that are being debugged have unique names. Possible research
// required to make sure this assumption is safe to make
// Language compilers may do automatic namespacing
impl<L, T> CodeFile<L, T> where L: Language, T: Transport {

    pub fn new(language: L, path: PathBuf, client: web3::Web3<T>) -> Result<Self, Error> {
        let name = path.file_name()
            .ok_or(LanguageError::NotFound(NotFoundError::File))?
            .to_str()
            .ok_or(LanguageError::InvalidPath)?
            .to_owned();

        if path.is_dir() {
            return Err(LanguageError::NotFound(NotFoundError::File)).map_err(|e| e.into());
        }
        let files: Vec<ContractFile<T>> = language.compile(path, &client.eth())?;
        Ok(Self { language, client, files, name })
    }


    /// find the first contract with name `contract`
    fn find_contract(&self, contract: &str) -> Result<&Contract<T>, LanguageError> {
        self.files
            .iter()
            .filter_map(|f| f.contract_by(|c| c.name == contract))
            .find(|c| c.name == contract)
            .ok_or(LanguageError::NotFound(NotFoundError::Contract))
    }

    /// Find the root contract that is being debugged
    pub fn root_name(&self) -> &str {
        &self.name
    }

    // passthrough for Source Map Trait
    /// Get a byte offset from a line number
    pub fn position_from_lineno(&self, lineno: usize, contract: &str) -> Result<Offset, Error> {
        let contract = self.find_contract(contract)?;
        contract.source_map().position_from_lineno(lineno)
    }

    /// Get a line number from a byte offset
    pub fn lineno_from_position(&self, offset: usize, contract: &str) -> Result<LineNo, Error> {
        let contract = self.find_contract(contract)?;
        contract.source_map().lineno_from_position(offset).map_err(|e| e.into())
    }

    pub fn current_line(&self, offset: usize) -> Result<Line, Error> {
        unimplemented!();
    }

    pub fn last_lines(&self, offset: usize, count: usize) -> Result<Vec<Line>, Error> {
        unimplemented!();
    }

    pub fn next_lines(&self, offset: usize, count: usize) -> Result<Vec<Line>, Error> {
        unimplemented!();
    }
}
