//! Codefile represents one source code file and all of the files it imports
use super::{Language, SourceMap, ContractFile, Contract, err::LanguageError};
use web3::Transport;
use std::path::{Path, PathBuf};

// every CodeFile is associated with a language
pub struct CodeFile<L: Language, T: Transport> {
    language: L,
    client: web3::Web3<T>,
    name: String,
    files: Vec<ContractFile<T>>,
}

impl<L, T> CodeFile<L, T> where L: Language, T: Transport {

    pub fn new(language: L, path: PathBuf, client: web3::Web3<T>) -> Result<Self, LanguageError> {
        let name = path.file_name()
            .ok_or(LanguageError::FileNotFound)?
            .to_str()
            .ok_or(LanguageError::InvalidPath)?
            .to_owned();

        if path.is_dir() {
            return Err(LanguageError::FileNotFound);
        }
        let files: Vec<ContractFile<T>> = language.compile(path, &client.eth())?;
        Ok(Self { language, client, files, name })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
/*
    pub fn contract(&self, name: &str) -> Option<&Contract<T>> {
        self.language
            .contracts()
            .find(|c| {
                c.name == name
            })
    }
    */
}
