use lunarity::ast::{Program};
use super::err::SolidityError;

// TODO: Make error type defined on traits generic
use crate::{ err::LanguageError, Ast};

pub struct SolidityAst<'ast> {
    program: Program<'ast>
}

impl<'ast> SolidityAst<'ast> {
    pub fn new(source: &str) -> Result<Self, SolidityError> {
        let program = lunarity::parse(source)
            .map_err(|e| SolidityError::AstParse(format!("{:?}", e)))?;
        Ok(Self { program })
    }
}


impl<'ast> Ast for SolidityAst<'ast> {
    type Err = LanguageError;

    fn contract_by_offset(&self, offset: u32) -> Result<String, Self::Err> {
        unimplemented!();
    }
}
