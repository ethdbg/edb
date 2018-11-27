use failure::Error;
use ethereum_types::Address;

use std::{
    path::PathBuf,
    str::FromStr
};

use crate::err::EDBError;

use edb_core::{Language, CompiledFiles};


#[derive(Debug, Clone)]
pub struct File {
    path: PathBuf,
    file_type: FileType,
}

impl File {
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn file_type(&self) -> &FileType {
        &self.file_type
    }

    pub fn compile<L>(&self, lang: L, addr: &Address) -> Result<CompiledFiles, Error> where L: Language {
        lang.compile(self.path(), addr).map_err(|e| e.into())
    }
}

impl From<PathBuf> for File {
    fn from(path: PathBuf) -> File {
        let file_type: FileType = path.extension()
            .expect("File must have an extension")
            .to_str()
            .expect("Extension is Invalid UTF8")
            .parse()
            .expect("Parsing to FileType should never fail; qed");
            
        File { path, file_type }    
    }
}

#[derive(Debug, Clone)]
pub enum FileType {
    Solidity,
    Vyper,
    LLL,
    Bamboo,
    Serpent
}

impl FromStr for FileType {
    type Err = Error;
    fn from_str(s: &str) -> Result<FileType, Error> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "sol" => Ok(FileType::Solidity),
            "vy"  => Ok(FileType::Vyper),
            "lll" => Ok(FileType::LLL),
            "bmb" => Ok(FileType::Bamboo), // TODO: don't know if this is the actual extension used
            "sp"  => Ok(FileType::Serpent), // TODO: Don't know if this is the actual extension used
            _     => Err(EDBError::FileExtensionParse(s).into())
        } 
    }
}

