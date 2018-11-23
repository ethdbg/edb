//! Codefile represents one source code file and all of the files it imports
use super::{Language, Line, OpcodeOffset, CharOffset, LineNo, contract::{Contract, ContractFile}, err::{LanguageError, NotFoundError}};
use failure::Error;
use web3::{Transport, types::Address};
use std::{path::{PathBuf}, rc::Rc};

// every CodeFile is associated with a language

pub struct CodeFile<L: Language, T: Transport> {
    language: L,
    client: web3::Web3<T>,
    name: String,
    files: Vec<Rc<ContractFile>>,
    // every Contract contains a reference back to the file (and therefore AST) from where it originated
    contracts: Vec<Contract<T>>
}

// TODO: Assumes all contracts that are being debugged have unique names. Possible research
// required to make sure this assumption is safe to make
// Language compilers may do automatic namespacing
impl<L, T> CodeFile<L, T> where L: Language, T: Transport {

    /// Create a new instance of Code File
    pub fn new(language: L, path: PathBuf, client: web3::Web3<T>, address: &Address) -> Result<Self, Error> {
        let name = path.file_name()
            .ok_or(LanguageError::NotFound(NotFoundError::File))?
            .to_str()
            .ok_or(LanguageError::InvalidPath)?
            .to_owned();

        if path.is_dir() {
            return Err(LanguageError::NotFound(NotFoundError::File)).map_err(|e| e.into());
        }
        let (files, contracts) = language.compile(path, &client.eth(), address)?;
        Ok(Self { language, client, files, contracts, name })
    }


    /// find the first contract with name `contract`
    pub fn find_contract(&self, contract: &str) -> Result<&Contract<T>, LanguageError> {
        self.contracts
            .iter()
            .find(|c| c.name() == contract)
            .ok_or(LanguageError::NotFound(NotFoundError::Contract))
    }

    /// Find the root contract that is being debugged
    pub fn root_name(&self) -> &str {
        &self.name
    }

    pub fn unique_exists(&self, lineno: LineNo, contract: &str) -> Result<bool, Error> {
        let contract = self.find_contract(contract)?;
        Ok(contract.source_map().unique_exists(lineno))
    }

    pub fn unique_opcode_pos(&self, lineno: LineNo, contract: &str) -> Result<OpcodeOffset, Error> {
        let contract = self.find_contract(contract)?;
        contract.source_map().unique_opcode_pos(lineno)
    }

    // passthrough for Source Map Trait
    /// Get a byte offset in the bytecode from a line number
    pub fn opcode_pos_from_lineno(&self, lineno: LineNo, from: OpcodeOffset, contract: &str) -> Result<OpcodeOffset, Error> {
        let contract = self.find_contract(contract)?;
        contract.source_map().opcode_pos_from_lineno(lineno, from)
    }

    pub fn char_pos_from_lineno(&self, lineno: LineNo, contract: &str) -> Result<CharOffset, Error> {
        let contract = self.find_contract(contract)?;
        contract.source_map().char_pos_from_lineno(lineno)
    }

    pub fn lineno_from_char_pos(&self, offset: CharOffset, contract: &str) -> Result<LineNo, Error> {
        let contract = self.find_contract(contract)?;
        contract.source_map().lineno_from_char_pos(offset)
    }

    pub fn lineno_from_opcode_pos(&self, offset: OpcodeOffset, contract: &str) -> Result<LineNo, Error> {
        let contract = self.find_contract(contract)?;
        contract.source_map().lineno_from_opcode_pos(offset)
    }

    pub fn current_line(&self, offset: usize, contract: &str) -> Result<Line, Error> {
        let contract = self.find_contract(contract)?;
        contract.source_map().current_line(offset).map_err(|e| e.into())
    }

    pub fn last_lines(&self, offset: usize, count: usize, contract: &str) -> Result<Vec<Line>, Error> {
        let contract = self.find_contract(contract)?;
        contract.source_map().last_lines(offset, count).map_err(|e| e.into())
    }

    pub fn next_lines(&self, offset: usize, count: usize, contract: &str) -> Result<Vec<Line>, Error> {
        let contract = self.find_contract(contract)?;
        contract.source_map().next_lines(offset, count).map_err(|e| e.into())
    }
}
