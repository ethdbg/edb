//! General types for EDB

use std::str::FromStr;

use super::{
    Configuration,
    cli::CLIArgs,
};

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
// |          Conf Types             |
// |                                 |
// |---------------------------------|


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
