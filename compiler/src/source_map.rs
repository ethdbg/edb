use super::{SourceMap, Language};

use std::{
    boxed::Box,
    str::FromStr,
};

use codespan::{ CodeMap, FileMap, ByteIndex, LineIndex };

/// Functions for Bytecode source map
pub struct BytecodeSourceMap<T: Language> {
    code_map: CodeMap,
    map: Vec<Box<dyn SourceMap>>,
}


/// struct representing one bytecode instruction and it's position in the source code
pub struct Instruction {
    /// Start Byte  offset in source
    pub start: usize,
    /// Length of code in source
    pub length: usize,
    /// Index of file in Solidity Compiler Output
    pub source_index: SourceIndex,
    /// Type of jump, if any
    pub jump: Jump,
    /// Position of this in bytecode (as an offset)
    pub position: usize,
}

impl From<(usize, usize, SourceIndex, Jump, usize)> for Instruction {
    fn from(values: (usize, usize, SourceIndex, Jump, usize)) -> Instruction {
        Instruction {
            start: values.0,
            length: values.1,
            source_index: values.2,
            jump: values.3,
            position: values.4
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SourceIndex {
    NoSource,
    Source(usize)
}

impl Default for SourceIndex {
    fn default() -> SourceIndex {
        SourceIndex::NoSource
    }
}

impl FromStr for SourceIndex {
    type Err = SourceMapVariant;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-1" => Ok(SourceIndex::NoSource),
            _ => Ok(SourceIndex::Source(s.parse()?))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Jump {
    IntoFunc,
    ReturnFunc,
    NormJump,
}

impl Default for Jump {
    fn default() -> Jump {
        Jump::NormJump
    }
}

impl ToString for Jump {
    fn to_string(&self) -> String {
        match self {
            Jump::IntoFunc   => "i".to_string(),
            Jump::ReturnFunc => "o".to_string(),
            Jump::NormJump   => "-".to_string(),
        }
    }
}

impl FromStr for Jump {
    type Err = SourceMapVariant;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i" => Ok(Jump::IntoFunc),
            "o" => Ok(Jump::ReturnFunc),
            "-" => Ok(Jump::NormJump),
            _ => {
                Err(SourceMapVariant::UnknownJumpVariant)
            }
        }
    }
}
