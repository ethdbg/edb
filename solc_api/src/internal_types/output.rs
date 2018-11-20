//! Output Types for Solidities Standard JSON
use std::{
    self,
    collections::{HashMap, hash_map},
    slice::Iter,
    str::FromStr
};
use serde_derive::*;
use serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess};
use { ethabi, hex };

use err::SolcApiError;

/// name of the file, including extension
type FileName = String;

pub struct CompiledSource {
    /// Compiled Source File
    sources: HashMap<FileName, CompiledSourceFile>,
    /// Contracts
    contracts: Vec<Contract>,
}

impl<'a> CompiledSource {
    pub(crate) fn new(raw: RawCompiledSource) -> Self {

        let contracts = raw.contracts
            .into_iter()
            .flat_map(|(k, v)| {
                v
                    .into_iter()
                    .map(|(inner_k, inner_v)| {
                        Contract {
                            file_name: k.to_string(),
                            name: inner_k.to_string(),
                            abi: inner_v.abi,
                            metadata: inner_v.metadata,
                            userdoc: inner_v.userdoc,
                            devdoc: inner_v.devdoc,
                            ir: inner_v.ir,
                            evm: inner_v.evm,
                            method_identifiers: inner_v.method_identifiers,
                            gas_estimates: inner_v.gas_estimates,
                            ewasm: inner_v.ewasm
                        }
                    }).collect::<Vec<Contract>>()
            })
            .collect::<Vec<Contract>>();

        Self {
            contracts,
            sources: raw.sources
        }
    }

    /// Iterate all contracts
    pub fn contracts(&self) -> Iter<Contract> {
        self.contracts.iter()
    }

    /// Iterate only the contracts that match the predicate
    pub fn contracts_by<F>(&self, fun: F) -> impl Iterator<Item = &Contract>
    where
        F: Fn(&Contract) -> bool,
    {

        self.contracts
            .iter()
            .filter(move |c| fun(c))
    }

    pub fn sources(&self) -> impl Iterator<Item = (&FileName, &CompiledSourceFile)> {
        self.sources.iter()
    }
}


#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RawCompiledSource {
    // TODO make a struct that fails compilation if any errors are returned
    /// Any Errors that occured during compilation
    errors: Option<Vec<Errors>>,
    sources: HashMap<String, CompiledSourceFile>,
    contracts: HashMap<String, HashMap<String, RawContract>>
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompiledSourceFile {
    pub id: usize,
    #[serde(skip_deserializing)]
    pub ast: Ast,
    #[serde(skip_deserializing)]
    pub legacy_ast: LegacyAst // Not Implemented
}

// TODO: Unimplemented Structs
#[derive(Debug, Clone, Default, Deserialize)]
/// Unimplemented!
pub struct LegacyAst;
#[derive(Debug, Clone, Default, Deserialize)]
/// Unimplemented!
pub struct Metadata;
#[derive(Debug, Clone, Default, Deserialize)]
/// Unimplemented!
pub struct UserDoc;
#[derive(Debug, Clone, Default, Deserialize)]
/// Unimplemented!
pub struct DevDoc;
#[derive(Debug, Clone, Default, Deserialize)]
/// Unimplemented!
pub struct LegacyAssembly;
#[derive(Debug, Clone, Default, Deserialize)]
/// Unimplemented!
pub struct MethodIdentifiers;
#[derive(Debug, Clone, Default, Deserialize)]
/// Unimplemented!
pub struct Ast;

#[derive(Debug, Clone, Deserialize)]
struct RawContract {
    /// The abi of the contract source
    abi: ethabi::Contract,
    #[serde(skip_deserializing)]
    /// Contract Metadata (Unimplemented)
    metadata: Option<Metadata>, // Unimplemented
    /// UserDoc (natspec) Unimplemented
    #[serde(skip_deserializing)]
    userdoc: Option<UserDoc>, // Unimplemented
    /// DevDoc (natspec) Unimplemented
    #[serde(skip_deserializing)]
    devdoc: Option<DevDoc>, // Unimplemented
    /// Intermediate Representation
    ir: Option<String>,
    /// Evm-related Outputs
    evm: Evm,
    /// List of Function Hashses (Unimplemented)
    method_identifiers: Option<MethodIdentifiers>,
    /// Function Gas Estimates
    gas_estimates: Option<GasEstimates>,
    #[serde(skip_deserializing)]
    ewasm: Option<EWasm>,
}

#[derive(Debug, Clone)]
pub struct Contract {
    /// Source file name
    pub file_name: String,
    /// Contract name
    pub name: String,
    /// Abi of the contract
    pub abi: ethabi::Contract,
    /// Contract Metadata (Unimplemented)
    pub metadata: Option<Metadata>, // Unimplemented
    /// UserDoc (natspec) Unimplemented
    pub userdoc: Option<UserDoc>, // Unimplemented
    /// DevDoc (natspec) Unimplemented
    pub devdoc: Option<DevDoc>, // Unimplemented
    /// Intermediate Representation
    pub ir: Option<String>,
    /// Evm-related Outputs
    pub evm: Evm,
    /// List of Function Hashses (Unimplemented)
    pub method_identifiers: Option<MethodIdentifiers>,
    /// Function Gas Estimates
    pub gas_estimates: Option<GasEstimates>,
    pub ewasm: Option<EWasm>,
}

/// eWasm related outputs
#[derive(Debug, Clone, Deserialize)]
pub struct EWasm {
    pub wast: String,
    pub wasm: String,
}

/// Gas Estimates of functions
#[derive(Debug, Clone, Deserialize)]
pub struct GasEstimates {
    /// Contract Creation
    pub creation: CreationGasEstimates,
    /// External <FunctionName, Cost>
    pub external: HashMap<String, String>,
    /// Internal <FunctionName, Cost>
    pub internal: HashMap<String, String>
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreationGasEstimates {
    pub code_deposit_cost: String,
    pub execution_cost: String,
    pub total_cost: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evm {
    /// Assembly Output
    pub assembly: Option<String>,
    #[serde(skip_deserializing)]
    /// Old Assembly Output (Unimplemented)
    pub legacy_assembly: Option<LegacyAssembly>, // Unimplemented
    /// Bytecode used when the contract is first committed to Ethereum
    pub bytecode: Option<Bytecode>,
    /// Bytecode used when called upon by transactions after deployment
    pub deployed_bytecode: Option<Bytecode>
}

#[derive(Debug, Clone)]
pub struct Bytecode {
    /// Bytecode as a hexstring
    pub object: Vec<u8>,
    /// Opcodes list (string)
    pub opcodes: Option<String>,
    /// Source Map (Decompressed)
    pub source_map: Vec<Instruction>,
    /// If given, this is an unlinked Object
    pub link_references: Option<HashMap<String, HashMap<String, Vec<Position>>>>
}

impl<'de> Deserialize<'de> for Bytecode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field { Object, Opcodes, SourceMap, LinkReferences };

        struct BytecodeVisitor;
        impl<'de> Visitor<'de> for BytecodeVisitor {
            type Value = Bytecode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Bytecode")
            }
            fn visit_map<V>(self, mut map: V) -> Result<Bytecode, V::Error>
            where
                V: MapAccess<'de>
            {
                let (mut object, mut opcodes, mut source_map, mut link_references) = (None, None, None, None);

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Object => {
                            if object.is_some() {
                                return Err(de::Error::duplicate_field("object"));
                            }
                            let code: String = map.next_value()?;
                            object = Some(hex::decode(code)
                                .map_err(|e| de::Error::custom(format!("{}", e)))?);
                        },
                        Field::Opcodes => {
                            if opcodes.is_some() {
                                return Err(de::Error::duplicate_field("opcodes"));
                            }
                            opcodes = Some(map.next_value()?);
                        },
                        Field::SourceMap => {
                            if source_map.is_some() {
                                return Err(de::Error::duplicate_field("sourceMap"));
                            }
                            source_map = Some(decompress(map.next_value()?)
                                .map_err(|e| de::Error::custom(format!("{}", e)))?);
                        },
                        Field::LinkReferences => {
                            if link_references.is_some() {
                                return Err(de::Error::duplicate_field("linkReferences"))
                            }
                            link_references = Some(map.next_value()?);
                        }
                    }
                }
                let object = object.ok_or_else(|| de::Error::missing_field("object"))?;
                // opcodes is an option already
                let source_map = source_map.ok_or_else(||de::Error::missing_field("sourceMap"))?;
                let link_references = link_references.ok_or_else(|| de::Error::missing_field("linkReferences"))?;

                Ok(Bytecode { object, opcodes, source_map, link_references })
            }
        }
        const FIELDS: &'static [&'static str] = &["object, opcodes, sourceMap, linkReferences"];
        deserializer.deserialize_struct("Bytecode", FIELDS, BytecodeVisitor)
    }
}



// RULES:
// If a field is empty, the value of the preceding element is used.
// If a : is missing, all following fields are considered empty.
//
// these are the same:
// 1:2:1  ;  1:9:1  ;  2:1:2  ;  2:1:2  ;  2:1:2
// 1:2:1  ;  :9     ;  2:1:2  ;         ;
fn decompress(source_map: &str) -> Result<Vec<Instruction>, SolcApiError> {
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

            let start = parts[0].parse().map_err(|e| SolcApiError::from(e));
            let length = parts[1].parse().map_err(|e| SolcApiError::from(e));
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
// used only to make the above collect::<_>() a bit more readable
type IterRes<T> = Result<T, SolcApiError>;

/// Struct representing s:l:f:j and a position -- the index in the bytecode
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Instruction {
    /// Start Byte  offset in source
    pub start: usize,
    /// Length of code in source
    pub length: usize,
    /// Index of file in Solidity Compiler Output
    pub source_index: SourceIndex,
    /// Type of jump, if any
    pub jump: Jump,
    /// Position in bytecode
    pub position: usize,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Instruction {} in source {} - {}. Jump: {:?}", self.position, self.start, self.start+self.length, self.jump)
    }
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
    type Err = SolcApiError;
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
    type Err = SolcApiError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i" => Ok(Jump::IntoFunc),
            "o" => Ok(Jump::ReturnFunc),
            "-" => Ok(Jump::NormJump),
            _ => {
                Err(SolcApiError::UnknownJumpVariant)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
/// Byte offsets into the bytecode. Linking replaces the 20 bytes located there.
pub struct Position {
    pub start: usize,
    pub length: usize,
}

// TODO: solc_api error type
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Errors {
    pub source_location: Option<SourceLocation>,
    #[serde(rename = "type")]
    pub variant: ErrorVariant,
    pub component: String,
    pub severity: String,
    pub message: String,
    pub formatted_message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ErrorVariant {
    SyntaxError,
    TypeError,
    InternalCompilerError,
    Exception,
    IOError,
    Warning,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub start: usize,
    pub end: usize,
}

#[cfg(test)]
mod tests {
    use super::{Jump, SourceIndex, Instruction};

    #[test]
    fn decompress_mappings() {
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
        assert_eq!(super::decompress(comp).unwrap(), de_comp);
    }
}
