use lunarity::{Program};
use super::err::SolidityError;

// TODO: Make error type defined on traits generic
use crate::{ err::LanguageError, Ast};


#[derive(Debug, Clone,)]
pub struct SolidityAst<'ast> {
    program: Program<'ast>
}

impl<'ast> SolidityAst<'ast> {
    pub fn new(source: &str) -> Result<Self, SolidityError> {
        let program = lunarity::parse(source)
            .map_err(|e| SolidityError::AstParse(format!("{:?}" e)))?;
        Self { program }
    }
}


impl<'ast> Ast for SolidityAst {
    type Err = LanguageError;

    fn contract_by_offset(&self, offset: u32) -> Result<String, Self::Err> {
        unimplemented!();
    }
}
