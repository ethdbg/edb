pub mod cli;
mod helpers;
use self::cli::{LogLevel, CLIArgs};

use failure::Error;
use fern::colors::{Color, ColoredLevelConfig};

use std::{
    path::PathBuf
};

pub struct Configuration {
    file: PathBuf,
    contract: Option<String>,
}

impl Configuration {
    pub fn new() -> Result<Self, Error> {
        let opts = self::cli::parse()?;
        init_logger(opts.log_level.clone().into());
        Ok(opts.into())
    }

    pub fn contract(&self) -> Option<&String> {
        self.contract.as_ref()
    }

    pub fn file(&self) -> &PathBuf {
        &self.file
    }
}


fn init_logger(level: log::LevelFilter) {
    let colors = ColoredLevelConfig::new()
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red)
        .debug(Color::Blue)
        .trace(Color::Magenta);
    let mut log_dir = dirs::data_local_dir()
        .expect("failed to find local data dir for logs");
    log_dir.push("edb");
    self::helpers::create_dir(log_dir.clone());
    log_dir.push("edb.logs");
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                    "{} [{}][{}] {} ::{};{}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    colors.color(record.level()),
                    message,
                    format_opt(record.file().map(|s| s.to_string())),
                    format_opt(record.line().map(|n| n.to_string()))
                ))
        })
        .chain(
            fern::Dispatch::new()
            .level(log::LevelFilter::Info)
            .level_for("edb_compiler", log::LevelFilter::Trace)
            .level_for("edb_emul", log::LevelFilter::Trace)
            .level_for("edb_core", log::LevelFilter::Trace)
            .level_for("edb", log::LevelFilter::Trace)
            .chain(fern::log_file(log_dir).expect("Failed to create edb.logs file"))
        )
        .chain(
            fern::Dispatch::new()
            .level(level)
            .chain(std::io::stdout())
        )
        .apply().expect("Could not init logging");
}

fn format_opt(file: Option<String>) -> String {
    match file {
        None => "".to_string(),
        Some(f) => f.to_string()
    }
}

// for now CLIArgs and Configuraition are the exact same struct
// However, this has a high possiblity of changing if a configuration file is introduced
// (which is planned)
// so these will remain as separate structs
impl From<CLIArgs> for Configuration {
    fn from(args: CLIArgs) -> Configuration {
        Configuration {
            file: args.file,
            contract: args.contract,
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
