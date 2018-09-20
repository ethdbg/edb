use super::{Language, contract::Contract};
use super::source_map::BytecodeSourceMap;
use std::{
    path::PathBuf,
};

//
pub struct CodeFile<T: Language> {
    source_maps: BytecodeSourceMap<T>,
    contracts: Vec<Contract>,
    source: String // only handling one source at first
}

impl<T> CodeFile<T> where T: Language {
    pub fn new(source: PathBuf) -> Self {
        unimplemented!()
    }
}
