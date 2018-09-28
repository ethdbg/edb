use failure::Fail;
use crate::err::LanguageError;

#[derive(Debug, Clone, PartialEq, Fail)]
pub enum SolidityError {
    #[fail(display = "Compiler Error")]
    Compiler,
    #[fail(display = "Error while parsing SourceFile for AST: {}", _0)]
    AstParse(String),
    #[fail(display = "IO Error")]
    Io(#[fail(cause)] std::io::Error),
    #[fail(display = "Parent directory not found; Path must not terminate in a root or prefix")]
    ParentNotFound,
}

impl From<std::io::Error> for SolidityError {
    fn from(err: std::io::Error) -> SolidityError {
        SolidityError::Io(err)
    }
}

impl From<SolidityError> for LanguageError {
    fn from(err: SolidityError) -> LanguageError {
        LanguageError::Language(Box::new(err))
    }
}
