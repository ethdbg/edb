use super::{Language, SourceMap, ContractFile, err::LanguageError};
use web3::{contract::Contract, Transport};
use std::path::{Path, PathBuf};

// every CodeFile is associated with a language
pub struct CodeFile<L: Language, T: Transport> {
    language: L,
    client: web3::Web3<T>,
    files: Vec<ContractFile<T>>,
}

impl<L, T> CodeFile<L, T> where L: Language, T: Transport {

    pub fn new(language: L, path: PathBuf, client: web3::Web3<T>) -> Result<Self, LanguageError> {
        let files: Vec<ContractFile<T>> = language.compile(path)?;
        Ok(Self { language, client, files })
    }
}
