//! Macros for Standard JSON Input for the Solidity Compiler

use serde_derive::*;
use serde::ser::{Serialize, Serializer};
use std::{
    collections::HashMap,
    path::PathBuf,
};
use ethereum_types::H160;
use crate::types::{Language, FoundationVersion};

/// Struct representing the Solidity Compilers' Standard JSON Input
#[derive(Serialize, Debug, Clone)]
struct StandardJson {
    language: Language,
    sources: HashMap<String, SourceFile>,
    settings: Settings,

}

impl StandardJson {
    fn load_defaults() -> Self {
        unimplemented!();
    }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct StandardJsonBuilder {
    /// specify the path of the source code
    source: PathBuf,
    /// a array of File Paths to source code
    urls: Vec<PathBuf>,
    /// EvmVersion to use
    version: Option<String>,
    /// [FLAG] whether to optimize output
    optimize: Option<bool>,
}

impl StandardJsonBuilder {
    // fs::canonicalize
    fn source_file(&mut self, val: PathBuf) -> &mut Self {
        let new = self;
        new.source = val;
        new
    }

    fn evm_version(&mut self, ver: FoundationVersion) -> &mut Self {
        let new = self;
        new.version = Some(ver.into());
        new
    }

    fn optimize(&mut self) -> &mut Self {
        let new = self;
        new.optimize = Some(true);
        new
    }

    fn build(&self) -> String { // returns standard JSON input for solidity compiler
        unimplemented!();
    }
}

#[derive(Serialize, Debug, Clone)]
struct SourceFile {
    /// Name of Source File and associated Info
    #[serde(rename = "keccak256")]
    hash: Option<String>,
    /// Paths to source files used in project
    urls: Option<Vec<PathBuf>>,
    /// Content of Source File. Required if urls not specified
    content: Option<String>
}

/// Optional additional settings to pass to the compiler
#[derive(Serialize, Debug, Clone)]
struct Settings {
    /// Optional: Sorted list of remappings
    remappings: Option<Vec<String>>,
    /// Optimizer Settings
    optimizer: Option<Optimizer>,
    /// EVM Version. Default Byzantium
    #[serde(rename = "evmVersion")]
    evm_version: Option<FoundationVersion>,
    /// Optional Metadata Settings
    metadata: Option<Metadata>,
    /// Addresses of the libraries. If not all libraries are given here, it can result in unlinked objects whose output data is different.
    /// The top level key is the the name of the source file where the library is used.
    /// If remappings are used, this source file should match the global path after remappings were applied.
    /// If this key is an empty string, that refers to a global level.
    libraries: Option<HashMap<String, HashMap<String, H160>>>,
    /// The following can be used to select desired outputs.
    /// If this field is omitted, then the compiler loads and does type checking, but will not generate any outputs apart from errors.
    /// The first level key is the file name and the second is the contract name, where empty contract name refers to the file itself,
    /// while the star refers to all of the contracts.
    ///
    /// Note that using a using `evm`, `evm.bytecode`, `ewasm`, etc. will select every
    /// target part of that output. Additionally, `*` can be used as a wildcard to request everything.
    ///
    /// Nested Hashmap -- First String is location/glob where contract is defined, second string is contract name/glob
    #[serde(rename = "outputSelection")]
    output_selection: HashMap<String, HashMap<String, Vec<SolcItem>>>,
}

/// Optimizer Settings
#[derive(Serialize, Debug, Clone)]
struct Optimizer {
    /// disabled by default
    enabled: bool,
    /// Optimize for how many times you intend to run the code.
    /// Lower values will optimize more for initial deployment cost, higher values will optimize more for high-frequency usage.
    runs: usize
}

/// Metadata Settings
#[derive(Serialize, Debug, Clone)]
struct Metadata {
    /// Use only literal content and not URLs (false by default)
    #[serde(rename = "useLiteralContent")]
    use_literal_content: bool,
}

/// OutputSelection Settings
#[derive(Debug, Clone)]
enum SolcItem {
    /// ABI
    Abi,
    /// AST of all source files
    Ast,
    /// Legacy AST of all source files
    LegacyAst,
    /// Developer Documentation (natspec)
    DevDoc,
    /// User Documentation (natspec)
    UserDoc,
    /// metadata
    Metadata,
    /// Ir - new assembly format before desugaring
    Ir,
    /// New Assembly Format after Desugaring
    Assembly,
    /// Old-style assembly format in JSON
    LegacyAssembly,
    /// The list of function hashses
    MethodIdentifiers,
    /// Function gas estimates
    GasEstimates,
    /// Bytecode (Evm Opt)
    Bytecode(EvmOpt),
    /// Deployed Bytecode Options
    DeployedBytecode(EvmOpt),
    /// eWASM s-expressions format (not currently supported)
    EwasmWast,
    /// eWASM binary format (not currently supported)
    EwasmWasm,
}

#[derive(Debug, Clone)]
enum EvmOpt {
    /// Bytecode Object
    BytecodeObject,
    /// Opcodes List
    Opcodes,
    /// Source Mapping
    SourceMap,
    /// Link References (if unlinked object)
    LinkReferences,
}

impl From<&EvmOpt> for String {
    fn from(val: &EvmOpt) -> String {
        match val {
            EvmOpt::BytecodeObject => "object".to_string(),
            EvmOpt::Opcodes        => "opcodes".to_string(),
            EvmOpt::SourceMap      => "sourceMap".to_string(),
            EvmOpt::LinkReferences => "linkReferences".to_string(),
        }
    }
}

impl Serialize for SolcItem {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        match self {
            SolcItem::Abi                   => serializer.serialize_str("abi"),
            SolcItem::Ast                   => serializer.serialize_str("ast"),
            SolcItem::LegacyAst             => serializer.serialize_str("legacyAST"),
            SolcItem::DevDoc                => serializer.serialize_str("devdoc"),
            SolcItem::UserDoc               => serializer.serialize_str("userdoc"),
            SolcItem::Metadata              => serializer.serialize_str("metadata"),
            SolcItem::Ir                    => serializer.serialize_str("ir"),
            SolcItem::Assembly              => serializer.serialize_str("evm.assembly"),
            SolcItem::LegacyAssembly        => serializer.serialize_str("evm.legacyAssembly"),
            SolcItem::Bytecode(opt)         => serializer.serialize_str(&format!("evm.bytecode.{}", String::from(opt))),
            SolcItem::DeployedBytecode(opt) => serializer.serialize_str(&format!("evm.deployedBytecode.{}", String::from(opt))),
            SolcItem::MethodIdentifiers     => serializer.serialize_str("evm.methodIdentifiers"),
            SolcItem::GasEstimates          => serializer.serialize_str("evm.gasEstimates"),
            SolcItem::EwasmWast             => serializer.serialize_str("ewasm.wast"),
            SolcItem::EwasmWasm             => serializer.serialize_str("ewasm.wasm"),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_opts() {
        let solc_items = vec![SolcItem::Abi, SolcItem::Bytecode(EvmOpt::BytecodeObject), SolcItem::DeployedBytecode(EvmOpt::BytecodeObject)];
        let ser_items = serde_json::to_string(&solc_items);
        println!("Ser Items: {:?}", ser_items);
    }
}
