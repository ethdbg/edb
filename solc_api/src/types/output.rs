//! Output Types for Solidities Standard JSON
use ethabi;
use std::{
    collections::HashMap,
};
use serde_derive::*;

#[derive(Debug, Clone, Deserialize)]
pub struct CompiledSource {
    pub errors: Option<Vec<Errors>>,
    pub sources: HashMap<String, CompiledSourceFile>,
    /// Contracts
    /// First key is source file, second key are the names of contracts included in that file
    pub contracts: HashMap<String, HashMap<String, Contract>>
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
pub struct LegacyAst;
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Metadata;
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserDoc;
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DevDoc;
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LegacyAssembly;
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MethodIdentifiers;
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Ast;

#[derive(Debug, Clone, Deserialize)]
pub struct Contract {
    pub abi: ethabi::Contract,
    #[serde(skip_deserializing)]
    /// Contract Metadata (Unimplemented)
    pub metadata: Option<Metadata>, // Unimplemented
    /// UserDoc (natspec) Unimplemented
    #[serde(skip_deserializing)]
    pub userdoc: Option<UserDoc>, // Unimplemented
    /// DevDoc (natspec) Unimplemented
    #[serde(skip_deserializing)]
    pub devdoc: Option<DevDoc>, // Unimplemented
    /// Intermediate Representation
    pub ir: Option<String>,
    /// Evm-related Outputs
    pub evm: Evm,
    /// List of Function Hashses (Unimplemented)
    method_identifiers: Option<MethodIdentifiers>,
    /// Function Gas Estimates
    gas_estimates: Option<GasEstimates>,
    #[serde(skip_deserializing)]
    ewasm: Option<EWasm>,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bytecode {
    /// Bytecode as a hexstring
    pub object: String,
    /// Opcodes list (string)
    pub opcodes: Option<String>,
    /// Compressed SourceMap
    pub source_map: String,
    /// If given, this is an unlinked Object
    pub link_references: Option<HashMap<String, HashMap<String, Vec<Position>>>>
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
