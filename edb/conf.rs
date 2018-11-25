mod cli;
mod helpers;
mod err;
mod types;

pub use self::types::*;

use failure::Error;
use ethereum_types::Address;

pub struct Configuration {
    pub file: File,
    transport: http::uri::Uri,
    contract: Option<String>,
    mode: Mode,
    address: Address,
}

impl Configuration {
    pub fn new() -> Result<Self, Error> {
        let opts = self::cli::parse()?;
        self::helpers::init_logger(opts.log_level.clone().into());
        Ok(opts.into())
    }

    pub fn contract(&self) -> Option<&String> {
        self.contract.as_ref()
    }

    pub fn file(&self) -> &File {
        &self.file
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn transport(&self) -> &http::uri::Uri {
        &self.transport
    }

    pub fn addr(&self) -> &Address {
        &self.address
    }
}


