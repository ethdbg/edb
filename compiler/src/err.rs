use failure::Fail;
use std;
// TODO: Complete errors
#[derive(Fail, Debug)]
pub enum CompilerError {
    #[fail(display = "Decode Error")]
    Decoding,
    #[fail(display = "Compile Error")]
    Compile,
    #[fail(display = "Error while Generating Source Map")]
    SourceMap
}

impl From<std::num::ParseIntError> for CompilerError {
    fn from(_err: std::num::ParseIntError) -> CompilerError {
        CompilerError::Decoding
    }
}
