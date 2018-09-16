//! Input Types for Solidity Standard Json API
use serde_derive::*;
use serde::ser::{Serialize, Serializer};
use crate::types::{Language, FoundationVersion};
use ethereum_types::H160;
use std::{
    collections::HashMap,
    path::PathBuf,
};

/// Struct representing the Solidity Compilers' Standard JSON Input
#[derive(Serialize, Debug, Clone, Default)]
pub struct StandardJson {
    /// Language (Solidity, serprent, LLL, etc)
    pub language: Language,
    /// Source Files
    pub sources: HashMap<String, SourceFile>,
    /// Compilation Settings
    pub settings: Settings,
}
#[derive(Debug, Clone, Serialize)]
pub struct UrlType(#[serde(serialize_with="url_ser")] pub url::Url);
#[derive(Serialize, Debug, Clone, Default)]
pub struct SourceFile {
    /// Name of Source File and associated Info
    #[serde(rename = "keccak256", skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Paths to source files used in project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<PathBuf>>,
    /// Content of Source File. Required if urls not specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>
}

fn url_ser<S>(url: &url::Url, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    serializer.serialize_str(url.as_str())
}

/// Optional additional settings to pass to the compiler
#[derive(Serialize, Debug, Clone)]
pub struct Settings {
    /// Optional: Sorted list of remappings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remappings: Option<Vec<String>>,
    /// Optimizer Settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer: Option<Optimizer>,
    /// EVM Version. Default Byzantium
    #[serde(rename = "evmVersion", skip_serializing_if = "Option::is_none")]
    pub evm_version: Option<FoundationVersion>,
    /// Optional Metadata Settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// Addresses of the libraries. If not all libraries are given here, it can result in unlinked objects whose output data is different.
    /// The top level key is the the name of the source file where the library is used.
    /// If remappings are used, this source file should match the global path after remappings were applied.
    /// If this key is an empty string, that refers to a global level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub libraries: Option<HashMap<String, HashMap<String, H160>>>,
    /// The following can be used to select desired outputs.
    /// If this field is omitted, then the compiler loads and does type checking, but will not generate any outputs apart from errors.
    /// The first level key is the file name and the second is the contract name, where empty contract name refers to the file itself,
    /// while the star refers to all of the contracts.
    ///
    /// Note that using a using `evm`, `evm.bytecode`, `ewasm`, etc. will select every
    /// target part of that output. Additionally, `*` can be used as a wildcard to request everything.
    ///
    /// Nested Hashmap -- First String is location/glob where contract is defined, second string is contract name/glob
    #[serde(rename = "outputSelection", skip_serializing_if = "Option::is_none")]
    pub output_selection: Option<HashMap<String, HashMap<String, Vec<SolcItem>>>>,
}

impl Default for Settings {
    fn default() -> Settings {
        let mut item = HashMap::new();
        item.insert("*".to_string(), vec![SolcItem::Abi, SolcItem::Ast,
                              SolcItem::DeployedBytecode(EvmOpt::BytecodeObject),
                              SolcItem::DeployedBytecode(EvmOpt::SourceMap)]);
        let mut output = HashMap::new();
        output.insert("*".to_string(), item);
        Settings {
            remappings: None,
            optimizer: None,
            evm_version: None,
            metadata: None,
            libraries: None,
            output_selection: Some(output)
        }
    }
}

/// Optimizer Settings
#[derive(Serialize, Debug, Clone)]
pub struct Optimizer {
    /// disabled by default
    pub enabled: bool,
    /// Optimize for how many times you intend to run the code.
    /// Lower values will optimize more for initial deployment cost, higher values will optimize more for high-frequency usage.
    pub runs: usize
}

/// Metadata Settings
#[derive(Serialize, Debug, Clone)]
pub struct Metadata {
    /// Use only literal content and not URLs (false by default)
    #[serde(rename = "useLiteralContent")]
    pub use_literal_content: bool,
}

/// OutputSelection Settings
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum SolcItem {
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

#[derive(Debug, Clone, PartialEq)]
pub enum EvmOpt {
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
