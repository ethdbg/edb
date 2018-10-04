use failure::Fail;
use crate::err::LanguageError;

#[derive(Debug, Fail)]
pub enum SolidityError {
    #[fail(display = "Compiler Error")]
    Compiler,
    #[fail(display = "Error while parsing SourceFile for AST: {}", _0)]
    AstParse(String),
    #[fail(display = "IO Error")]
    Io(#[fail(cause)] std::io::Error),
    #[fail(display = "Parent directory not found; Path must not terminate in a root or prefix")]
    ParentNotFound,
    #[fail(display = "Source Mapping Error {}", _0)]
    SourceMap(#[cause] SourceMapError)
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

#[derive(Debug, Fail)]
pub enum SourceMapError {
    #[fail(display = "Did not find line number that corresponded to offset provided")]
    LineNotFound,
    #[fail(display = "No offset found for line number provided")]
    OffsetNotFound,
    #[fail(display = "Number of last lines to display is greater than current line number")]
    CountOutOfBounds,
}
