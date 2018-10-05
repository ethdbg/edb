use crate::{ SourceMap, Line, LineNo, Offset, map::Map };
use std::{iter::FromIterator, collections::HashMap };
use super::err::{SolidityError, SourceMapError};
use solc_api::types::Instruction;
use log::*;
use failure::Error;

// TODO many of these Strings, Vec<> can be made references with a lifetime on ContractFile
pub struct SoliditySourceMap {
    /// simple map of the source itself
    map: Map,
    /// Source map acquired from Solidity Compiler
    program_map: Vec<Instruction>,
    line_cache: HashMap<usize, Option<usize>>

}

impl SoliditySourceMap {

    pub fn new(src: &str, source_map: Vec<Instruction>) -> Self {
        let mut cache = HashMap::new();
        let map = Map::new(src);
        for line in src.lines().enumerate() {
            cache.insert(line.0, Self::shortest_len(&source_map, &map, line.0).map(|i| i.position));
        }

        Self {
            map: Map::new(src),
            program_map: source_map,
            line_cache: cache
        }
    }

    // find the instructions with shortest length, and returns the line number that contains that
    // offset
    fn shortest_len<'a>(prog_map: &'a Vec<Instruction>, map: &Map, lineno: usize) -> Option<&'a Instruction> {
        let mut shortest = None;
        for inst in prog_map.iter() {
            shortest = match shortest {
                None => if map.find_line(inst.start).unwrap() == (lineno) { Some(inst) } else { None },
                Some(current) => {
                    if map.find_line(inst.start).unwrap() == (lineno) && inst.length < current.length {
                        Some(inst)
                    } else {
                        Some(current)
                    }
                }
            }
        }
        shortest
    }
}

impl SourceMap for SoliditySourceMap {

    fn position_from_lineno(&self, lineno: usize) -> Result<Offset, Error> {
        Ok(self.line_cache.get(&lineno)
            .ok_or(SolidityError::SourceMap(SourceMapError::OffsetNotFound))?
            .ok_or(SolidityError::SourceMap(SourceMapError::OffsetNotFound))?)
    }

    fn lineno_from_position(&self, offset: usize) -> Result<LineNo, Error> {
        Ok(self.map.find_line(offset).ok_or(SolidityError::SourceMap(SourceMapError::LineNotFound))?)
    }

    fn current_line(&self, offset: usize) -> Result<Line, Error> {
        let line = self.map.find_line(offset).ok_or(SolidityError::SourceMap(SourceMapError::LineNotFound))?;
        let line_str = self.map.line(line)?;
        Ok((line, String::from_iter(line_str)))
    }

    fn last_lines(&self, offset: usize, count: usize) -> Result<Vec<Line>, Error> {
        let line = self.map.find_line(offset).ok_or(SolidityError::SourceMap(SourceMapError::LineNotFound))?;
        if count > line {
            return Err(SolidityError::SourceMap(SourceMapError::CountOutOfBounds)).map_err(|e| e.into());
        }
        Ok(self.map
           .lines((line - count)..line)?
           .into_iter()
           .enumerate()
           .collect::<Vec<Line>>())
    }

    fn next_lines(&self, offset: usize, count: usize) -> Result<Vec<Line>, Error> {
        let line = self.map.find_line(offset).ok_or(SolidityError::SourceMap(SourceMapError::LineNotFound))?;
        if count > (self.map.len() - line) {
            return Err(SolidityError::SourceMap(SourceMapError::CountOutOfBounds)).map_err(|e| e.into());
        }

        Ok(self.map
           .lines(line..(line + count))?
           .into_iter()
           .enumerate()
           .collect::<Vec<Line>>())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

}

