use failure::Fail;
use std;

#[derive(Fail, Debug, Clone, PartialEq)]
pub enum SolcApiError {
    #[fail(display = "Failed to Decompress the SourceMap")]
    FailedToDecompress(#[cause] std::num::ParseIntError),
    #[fail(display = "Unknown Jump Variant in Compressed Sourcemap")]
    UnknownJumpVariant,
}


impl From<std::num::ParseIntError> for SolcApiError {
    fn from(err: std::num::ParseIntError) -> SolcApiError {
        SolcApiError::FailedToDecompress(err)
    }
}


