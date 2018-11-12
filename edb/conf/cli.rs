use std::{
    path::PathBuf,
    str::FromStr
};
use failure::Error;
use clap::{App, load_yaml};
use ethereum_types::H160;


#[derive(Debug, Clone)]
pub enum LogLevel {
    None, // Error by default
    Info,
    Debug,
    Insane
}

pub struct CLIArgs {
    pub file: PathBuf,
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

    Ok(CLIArgs { file, contract, log_level, address })
}

