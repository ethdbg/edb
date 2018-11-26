use log::*;
use failure::Error;

use std::{
    path::PathBuf,
    str::FromStr
};

use super::{
    Configuration,
    cli::CLIArgs,
    err::ConfigurationError
};

use edb_core::{Language};

// -----------------------------------
// |          CLI Types              |
// |                                 |
// |---------------------------------|

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    None, // Error by default
    Info,
    Debug,
    Insane
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Rpc,
    Tui
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::Tui
    }
}

impl FromStr for Mode {
    type Err = ();
    fn from_str(s: &str) -> Result<Mode, ()> { // cannot fail because of default
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "rpc" => Ok(Mode::Rpc),
            "tui" => Ok(Mode::Tui),
            _ => Ok(Mode::default()),
        }
    }
}

// for now CLIArgs and Configuraition are the exact same struct
// However, this has a high possiblity of changing if a configuration file is introduced
// (which is planned)
// so these will remain as separate structs
impl From<CLIArgs> for Configuration {
    fn from(args: CLIArgs) -> Configuration {
        Configuration {
            file: args.file.into(),
            contract: args.contract,
            transport: args.transport,
            mode: args.mode,
            address: args.address
        }
    }
}

// -----------------------------------
// |       Conf  Types               |
// |                                 |
// |---------------------------------|

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

    pub fn compile<L>(&self, lang: L, addr: &Address) -> Result<Compiled, Error> where L: Language {
        lang.compile(lang, addr).map_err(|e| e.into())
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
            _     => Err(ConfigurationError::FileExtensionParse(s).into())
        } 
    }
}


impl From<LogLevel> for log::LevelFilter {
    fn from(log_level: LogLevel) -> log::LevelFilter {
        match log_level {
            LogLevel::None => log::LevelFilter::Error,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Insane => log::LevelFilter::Trace,
        }
    }
}
