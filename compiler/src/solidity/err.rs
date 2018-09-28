use failure::Fail;


#[derive(Debug, Clone, PartialEq, Fail)]
pub enum SolidityError {
    #[fail(display = "Compiler Error")]
    Compiler,
    #[fail(display = "Error while parsing SourceFile for AST: {}", _0)]
    AstParse(String),
    #[fail(display = "IO Error")]
    Io(#[fail(cause)] std::io::Error),
}

impl From<std::io::Error> for SolidityError {
    fn from(err: std::io::Error) -> SolidityError {
        SolidityError::Io(err)
    }
}
