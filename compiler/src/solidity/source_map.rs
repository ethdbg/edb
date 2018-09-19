//! Source Mapping Decoding and tools for Solidity
use std::{
    str::FromStr,
    string::ToString,
};
use super::err::{SolidityError, SourceMapVariant};

use log::*;

// TODO: Write a custom deserialize to automatically put decompressed map in CompiledSource
//     - that would probably get rid of this entire file, and would be more intuitive

// s:l:f:j
// s = start of range in source_file
// l = length of source range
// f = source index (integer identifier to refer to a source file)
// j = Jump Instruction

/// Struct Representing a Source Map for Solidity
#[derive(Debug)]
pub struct SoliditySourceMap {
    /// Decompressed Mapping. Each element represents an instruction, and it's position in the source code
    /// IE: decompressed_map[1] == Instruction 1
    pub instructions: Vec<Instruction>,
}

/// Struct representing s:l:f:j
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Instruction {
    pub start: usize,
    pub length: usize,
    pub source_index: SourceIndex,
    pub jump: Jump,
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

impl SoliditySourceMap {
    pub fn new(source_map: &str) -> Result<Self, SourceMapVariant> {
        Ok(SoliditySourceMap {
            instructions: Self::decompress(source_map)?,
        })
    }

    // RULES:
    // If a field is empty, the value of the preceding element is used.
    // If a : is missing, all following fields are considered empty.
    //
    // these are the same:
    // 1:2:1  ;  1:9:1  ;  2:1:2  ;  2:1:2  ;  2:1:2
    // 1:2:1  ;  :9     ;  2:1:2  ;         ;
    fn decompress(source_map: &str) -> Result<Vec<Instruction>, SourceMapVariant> {
        let mut last_ele: [&str; 4] = [""; 4];
        let values = source_map
            .split(';')
            .enumerate()
            .map(|(idx, ele)| {
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

                let start = parts[0].parse().map_err(|e| SourceMapVariant::from(e));
                let length = parts[1].parse().map_err(|e| SourceMapVariant::from(e));
                let source_index = parts[2].parse();
                let jump = parts[3].parse();
                (start, length, source_index, jump, idx)
            })
            .collect::<Vec<(IterRes<usize>, IterRes<usize>, IterRes<SourceIndex>, IterRes<Jump>, usize)>>();
        let mut instructions: Vec<Instruction> = Vec::new();
        for instruction in values.into_iter() {
            instructions.push(Instruction {
                start: instruction.0?,
                length: instruction.1?,
                source_index: instruction.2?,
                jump: instruction.3?,
                position: instruction.4
            });
        }
        Ok(instructions)
    }
}

// used only to make the above collect::<_>() a bit more readable
type IterRes<T> = Result<T, SourceMapVariant>;
#[cfg(test)]
mod test {
    use log::*;
    use super::*;
    use speculate::speculate;
    use pretty_env_logger;
    // 1:2:1  ;  1:9:1  ;  2:1:2  ;  2:1:2  ;  2:1:2
    // 1:2:1  ;  :9     ;  2:1:2  ;         ;
    // 1:2:1:o;  :9     ;  2:1:2:-;         ;
    speculate! {
        describe "source map" {
            before {
                #[allow(unused_must_use)] {
                    pretty_env_logger::try_init();
                }
            }

            it "should decompress mappings" {
                let comp = "1:2:1:o;:9;2:1:2:-;;";
                let de_comp = vec![
                    Instruction {
                        start: 1,
                        length: 2,
                        source_index: SourceIndex::Source(1),
                        jump: Jump::ReturnFunc,
                        position: 0,
                    },
                    Instruction {
                        start: 1,
                        length: 9,
                        source_index: SourceIndex::Source(1),
                        jump: Jump::ReturnFunc,
                        position: 1,
                    },
                    Instruction {
                        start: 2,
                        length: 1,
                        source_index: SourceIndex::Source(2),
                        jump: Jump::NormJump,
                        position: 2,
                    },
                    Instruction {
                        start: 2,
                        length: 1,
                        source_index: SourceIndex::Source(2),
                        jump: Jump::NormJump,
                        position: 3,
                    },
                    Instruction {
                        start: 2,
                        length: 1,
                        source_index: SourceIndex::Source(2),
                        jump: Jump::NormJump,
                        position: 4,
                    },
                ];
                info!("Decompressed Mappings: {:?}", SoliditySourceMap::decompress(comp));
                assert_eq!(SoliditySourceMap::decompress(comp), de_comp);
            }
        }
    }
}
