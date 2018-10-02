use crate::{ SourceMap, Line, LineNo, Offset, map::Map };
use std::iter::FromIterator;
use super::err::{SolidityError, SourceMapError};
use solc_api::types::Instruction;
use failure::Error;

// TODO many of these Strings, Vec<> can be made references with a lifetime on ContractFile
pub struct SoliditySourceMap {
    /// simple map of the source itself
    map: Map,
    /// Source map acquired from Solidity Compiler
    program_map: Vec<Instruction>
}

impl SoliditySourceMap {

    pub fn new(src: &str, source_map: Vec<Instruction>) -> Self {
        Self {
            map: Map::new(src),
            program_map: source_map,
        }
    }

    // find the instructions with shortest length, and returns the line number that contains that
    // offset
    fn shortest_len(&self, lineno: usize) -> Option<&Instruction> {
        self.program_map
            .iter()
            .fold(None, |min, instruction| {
                match min {
                    None => if self.map.find_line(instruction.start)? == lineno { Some(instruction) } else { None },
                    Some(other) => {
                        if self.map.find_line(instruction.start)? == lineno && other.length < instruction.length {
                            Some(instruction)
                        } else {
                            Some(other)
                        }
                    }
                }
            })
    }
}

impl SourceMap for SoliditySourceMap {

    fn position_from_lineno(&self, lineno: usize) -> Result<Offset, Error> {
        Ok(self.shortest_len(lineno).ok_or(SolidityError::SourceMap(SourceMapError::OffsetNotFound))?.position)
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

