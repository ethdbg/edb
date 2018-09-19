use failure::Fail;


#[derive(Debug, Fail)]
pub enum SolidityError {
     #[fail(display = "Source Map Error") ]
    SourceMap(SourceMapVariant),
    #[fail(display = "Compiler Error")]
    CompilerError,
    #[fail(display = "IO Error")]
    Io(#[fail(cause)] std::io::Error),
}


impl From<std::io::Error> for SolidityError {
    fn from(err: std::io::Error) -> SolidityError {
        SolidityError::Io(err)
    }
}
#[derive(Debug, Clone, Fail)]
pub enum SourceMapVariant {
    #[fail(display = "Unknown Jump Variant; Could Not Decode Solidity Source Map")]
    UnknownJumpVariant,
    #[fail(display = "Failed to properly decode source mapping")]
    Decode(#[cause] std::num::ParseIntError),
}


impl From<std::num::ParseIntError> for SourceMapVariant {
    fn from(err: std::num::ParseIntError) -> SourceMapVariant {
        SourceMapVariant::Decode(err)
    }
}


impl From<SourceMapVariant> for SolidityError {
    fn from(err: SourceMapVariant) -> SolidityError {
        SolidityError::SourceMap(err)
    }
}
