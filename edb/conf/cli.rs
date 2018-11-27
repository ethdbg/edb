use std::{
    path::PathBuf,
    str::FromStr
};

use log::*;
use http::uri::Uri;
use failure::Error;
use clap::{App, load_yaml, value_t};
use ethereum_types::H160;

use super::types::*;


pub struct CLIArgs {
    pub file: PathBuf,
    pub mode: Mode,
    pub transport: Uri,
    pub contract: Option<String>,
    pub log_level: LogLevel,
    pub address: H160,
}

pub fn parse() -> Result<CLIArgs, Error> {
    let yaml = load_yaml!("cli_args.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let contract = matches.value_of("contract").map(|c| c.to_owned());
    let file = matches.value_of("file").expect("File Argument is Required; qed");
    let file = PathBuf::from(&file);
    let log_level = match matches.occurrences_of("verbose") {
        0 => LogLevel::None,
        1 => LogLevel::Info,
        2 => LogLevel::Debug,
        3 | _ => LogLevel::Insane,
    };
    let address = H160::from_str(matches.value_of("address").expect("Missing Address"))?;
    let mode = value_t!(matches.value_of("mode"), Mode).ok().unwrap_or_else(|| {
        warn!("No RPC mode specified, using default 'TUI'");
        Mode::default()
    });
    let transport = matches.value_of("rpc").expect("Must specify an RPC to use");
    let transport = http::uri::Uri::from_shared(transport.as_bytes().into())?;

    Ok(CLIArgs { file, mode, transport, contract, log_level, address })
}

