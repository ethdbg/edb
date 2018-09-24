use failure::Fail;
use log::*;

#[derive(Fail, Debug, Clone)]
pub enum LanguageError {
    #[fail(display = "Could not obtain a Source Map")]
    SourceMap(#[cause] SourceMapError),
    #[fail(display = "")]
    NotFound(NotFoundError),
    #[fail(display = "An error occurrede while communicating with the local test node")]
    NodeIo(String),
    #[fail(display = "Could not parse source code file for line number positions")]
    ParseError
}

#[derive(Fail, Debug, Clone)]
pub enum NotFoundError {
    #[fail(display = "Contract not Found. Are you sure it is deployed to the specified testnet?")]
    Contract,
}

#[derive(Fail, Debug, Clone)]
pub enum SourceMapError {
    #[fail(display = "Unknown Jump Variant: {}", _0)]
    UnknownJump(String),
    #[fail(display = "Decode Error")]
    Decode(#[cause] std::num::ParseIntError),
}

impl From<web3::error::Error> for LanguageError {
    fn from(err: web3::error::Error) -> LanguageError {
        LanguageError::NodeIo(format!("{}", err))
    }
}

impl From<std::num::ParseIntError> for SourceMapError {
    fn from(err: std::num::ParseIntError) -> SourceMapError {
        SourceMapError::Decode(err)
    }
}


impl From<pest::error::Error<super::map::Rule>> for LanguageError {
    fn from(err: pest::error::Error<super::map::Rule>) -> LanguageError {
        error!("Fatal Internal Error Occurred. This is a bug within EDB. Please Report it: {}", err);
        trace!("{:?}", err);
        LanguageError::ParseError
    }
}
