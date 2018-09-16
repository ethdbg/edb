//! Output Types for Solidities Standard JSON

use std::{
    collections::HashMap,
};
use serde_derive::*;

#[derive(Debug, Clone, Deserialize)]
pub struct CompiledSource {
    pub errors: Option<Vec<Errors>>,
    pub sources: HashMap<String, CompiledSource>,
    /// Contracts
    /// First key is source file, second key are the names of contracts included in that file
    pub contracts: HashMap<String, HashMap<String, Contract>>
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompiledSourceFile {
    pub id: usize,
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
    ewasm: EWasm,
}

/// eWasm related outputs
#[derive(Debug, Clone, Deserialize)]
pub struct EWasm {
    wast: String,
    wasm: String,
}

/// Gas Estimates of functions
#[derive(Debug, Clone, Deserialize)]
pub struct GasEstimates {
    /// Contract Creation
    creation: CreationGasEstimates,
    /// External <FunctionName, Cost>
    external: HashMap<String, String>,
    /// Internal <FunctionName, Cost>
    internal: HashMap<String, String>
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreationGasEstimates {
    code_deposit_cost: String,
    execution_cost: String,
    total_cost: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evm {
    /// Assembly Output
    assembly: Option<String>,
    #[serde(skip_deserializing)]
    /// Old Assembly Output (Unimplemented)
    legacy_assembly: Option<LegacyAssembly>, // Unimplemented
    /// Bytecode used when the contract is first committed to Ethereum
    bytecode: Option<Bytecode>,
    /// Bytecode used when called upon by transactions after deployment
    deployed_bytecode: Option<Bytecode>
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bytecode {
    /// Bytecode as a hexstring
    object: Vec<u8>,
    /// Opcodes list (string)
    opcodes: Option<String>,
    /// Compressed SourceMap
    source_map: String,
    /// If given, this is an unlinked Object
    link_references: Option<HashMap<String, HashMap<String, Vec<Position>>>>
}

#[derive(Debug, Clone, Deserialize)]
/// Byte offsets into the bytecode. Linking replaces the 20 bytes located there.
pub struct Position {
    start: usize,
    length: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Errors {
    source_location: Option<SourceLocation>,
    #[serde(rename = "type")]
    variant: ErrorVariant,
    component: String,
    severity: String,
    message: String,
    formatted_message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ErrorVariant {
    TypeError,
    InternalCompilerError,
    Exception
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourceLocation {
    file: String,
    start: usize,
    end: usize,
}
