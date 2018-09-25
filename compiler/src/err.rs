use failure::Fail;

#[derive(Fail, Debug, Clone)]
pub enum LanguageError {
    #[fail(display = "Could not obtain a Source Map")]
    SourceMap(#[cause] SourceMapError),
    #[fail(display = "")]
    NotFound(NotFoundError),
    #[fail(display = "An error occurrede while communicating with the local test node")]
    NodeIo(String),
    #[fail(display = "Could not parse source code file for line number positions")]
    ParseError,
    #[fail(display = "Source path must be a file")]
    FileNotFound,
    #[fail(display = "Path must be valid UTF-8")]
    InvalidPath,
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

#[derive(Fail, Debug, Clone)]
pub enum MapError {
    #[fail(display = "Operation Out of Bounds of Source File")]
    OutOfBounds,
    #[fail(display = "Queried line is out-of-bounds of Source File")]
    LineOutOfBounds,
    #[fail(display = "Queried column is out-of-bounds of line")]
    ColOutOfBounds,
    #[fail(display = "Cannot get range from a function which returns a single integer")]
    CannotGetRange,
}
