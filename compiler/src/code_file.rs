use super::{Language, SourceMap, ContractFile, err::LanguageError};
use web3::{contract::Contract, Transport};
use std::path::{Path, PathBuf};

// every CodeFile is associated with a language
pub struct CodeFile<'a, L: Language<'a>, T: Transport> {
    language: L,
    client: web3::Web3<T>,
    name: String,
    files: Vec<ContractFile<'a, T>>,
}

impl<'a, L, T> CodeFile<'a, L, T> where L: Language<'a>, T: Transport {

    pub fn new(language: L, path: PathBuf, client: web3::Web3<T>) -> Result<Self, LanguageError> {
        let name = path.file_name()
            .ok_or(LanguageError::FileNotFound)?
            .to_str()
            .ok_or(LanguageError::InvalidPath)?
            .to_owned();
        let files: Vec<ContractFile<T>> = language.compile(path)?;
        Ok(Self { language, client, files, name })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
