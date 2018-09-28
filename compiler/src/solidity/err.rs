use failure::Fail;


#[derive(Debug, Fail)]
pub enum SolidityError {
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
