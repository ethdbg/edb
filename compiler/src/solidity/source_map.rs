//! Source Mapping Decoding and tools for Solidity
use std::{
    collections::HashMap,
    str::FromStr,
    string::ToString,
};
use crate::err::CompilerError;

use log::*;
use itertools::Itertools;

// TODO: Write a custom deserialize to automatically put decompressed map in CompiledSource
//     - that would probably get rid of this entire file, and would be more intuitive

// s:l:f:j
// s = start of range in source_file
// l = length of source range
// f = source index (integer identifier to refer to a source file)
// j = Jump Instruction

///Instruction Offset
type InstructionOffset = usize;

/// Struct Representing a Source Map for Solidity
pub struct SoliditySourceMap {
    /// Decompressed Mapping. Each element represents an instruction, and it's position in the source code
    /// IE: decompressed_map[1] == Instruction 1
    map: Vec<Instruction>,
}

/// Struct representing s:l:f:j
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Instruction {
    start: usize,
    length: usize,
    source_index: SourceIndex,
    jump: Jump,
}

#[derive(Clone, Debug, PartialEq)]
enum SourceIndex {
    NoSource,
    Source(usize)
}

impl Default for SourceIndex {
    fn default() -> SourceIndex {
        SourceIndex::NoSource
    }
}

impl FromStr for SourceIndex {
    type Err = CompilerError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-1" => Ok(SourceIndex::NoSource),
            _ => Ok(SourceIndex::Source(s.parse()?))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Jump {
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
    type Err = CompilerError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i" => Ok(Jump::IntoFunc),
            "o" => Ok(Jump::ReturnFunc),
            "-" => Ok(Jump::NormJump),
            _ => {
                error!("Unknown Jump Variant");
                Err(CompilerError::Decoding)
            }
        }
    }
}

impl SoliditySourceMap {
    pub fn new(source_map: &str) -> Self {
        SoliditySourceMap {
            map: Self::decompress(source_map),
        }
    }

    // RULES:
    // If a field is empty, the value of the preceding element is used.
    // If a : is missing, all following fields are considered empty.
    //
    // these are the same:
    // 1:2:1  ;  1:9:1  ;  2:1:2  ;  2:1:2  ;  2:1:2
    // 1:2:1  ;  :9     ;  2:1:2  ;         ;
    fn decompress(source_map: &str) -> Vec<Instruction> {
        let mut last_ele: [&str; 4] = [""; 4];
        source_map
            .split(';')
            .map(|ele| {
                let mut parts = ele
                    .split(':')
                    .enumerate()
                    .map(|(i,e)| {
                        if e == "" {last_ele[i]}
                        else {e}
                    })
                    .collect::<Vec<&str>>();
                last_ele.iter().enumerate().for_each(|(i, e)| {
                    if parts.get(i).is_none() {
                        parts.push(e);
                    }
                });
                assert_eq!(parts.len(), 4);
                last_ele = [parts[0], parts[1], parts[2], parts[3]];
                Instruction {
                    start: parts[0].parse().expect("Start could not be parsed!"),
                    length: parts[1].parse().expect("Length could not be parsed!"),
                    source_index: parts[2].parse().expect("Source Index could not be parsed!"),
                    jump: parts[3].parse().expect("Jump could not be parsed!")
                }
            })
            .collect::<Vec<Instruction>>()
    }
}


#[cfg(test)]
mod test {
    use speculate::speculate;
    use log::*;
    use super::*;
    // 1:2:1  ;  1:9:1  ;  2:1:2  ;  2:1:2  ;  2:1:2
    // 1:2:1  ;  :9     ;  2:1:2  ;         ;
    // 1:2:1:o;  :9     ;  2:1:2:-;         ;
    speculate! {
        describe "source map" {
            before {
                pretty_env_logger::try_init();
                // do nothing
            }

            it "should decompress mappings" {
                let comp = "1:2:1:o;:9;2:1:2:-;;";
                let de_comp = vec![
                    Instruction {
                        start: 1,
                        length: 2,
                        source_index: SourceIndex::Source(1),
                        jump: Jump::ReturnFunc
                    },
                    Instruction {
                        start: 1,
                        length: 9,
                        source_index: SourceIndex::Source(1),
                        jump: Jump::ReturnFunc,
                    },
                    Instruction {
                        start: 2,
                        length: 1,
                        source_index: SourceIndex::Source(2),
                        jump: Jump::NormJump,
                    },
                    Instruction {
                        start: 2,
                        length: 1,
                        source_index: SourceIndex::Source(2),
                        jump: Jump::NormJump,
                    },
                    Instruction {
                        start: 2,
                        length: 1,
                        source_index: SourceIndex::Source(2),
                        jump: Jump::NormJump,
                    },
                ];
                info!("Decompressed Mappings: {:?}", SoliditySourceMap::decompress(comp));
                assert_eq!(SoliditySourceMap::decompress(comp), de_comp);
            }
        }
    }
}
