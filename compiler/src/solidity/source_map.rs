use crate::{ SourceMap, Line, err::LanguageError, map::Map };

pub struct SoliditySourceMap {
    map: Map,
    source: String,
}

impl SoliditySourceMap {
    pub fn new(src: &str) -> Self {
        Self {
            map: Map::new(src),
            source: src.to_string()
        }
    }
}

impl SourceMap for SoliditySourceMap {
    type Err = LanguageError;

    fn position_from_lineno(&self, lineno: usize) -> usize {
        unimplemented!();
    }

    fn lineno_from_position(&self, offset: usize) -> usize {
        unimplemented!();
    }

    fn current_line(&self, offset: usize) -> Result<Line, Self::Err> {
        unimplemented!();
    }

    fn last_lines(&self, offset: usize, count: usize) -> Result<Vec<Line>, Self::Err> {
        unimplemented!();
    }

    fn next_lines(&self, offset: usize, count: usize) -> Result<Vec<Line>, Self::Err> {
        unimplemented!();
    }
}

