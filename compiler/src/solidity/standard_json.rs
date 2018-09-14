//! Macros for Standard JSON Input for the Solidity Compiler

use serde_derive::*;
use std::{
    collections::HashMap,
};
use ethereum_types::H160;
use super::{
    types::{Language, FoundationVersion},
};

/// Struct representing the Solidity Compilers' Standard JSON Input
#[derive(Serialize, Debug, Clone)]
struct StandardJson {
    language: Language,
    sources: HashMap<String, SourceFile>,
    settings: Settings,

}

impl StandardJson {
    fn load_defaults() -> Self {
        StandardJson {

        }
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

    fn optimize(&mut self) -> &mut self {
        let new = self;
        new.optimize = Some(true);
        new
    }

    fn build(&self) -> String { // returns standard JSON input for solidity compiler

    }
}

#[derive(Serialize, Debug, Clone)]
enum SourceFile {
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
enum Settings {
    /// Optional: Sorted list of remappings
    remappings: Option<Vec<String>>,
    /// Optimizer Settings
    optimizer: Option<Optimizer>,
    #[serde(rename = "evmVersion")]
    /// EVM Version. Default Byzantium
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
    output_selection: HashMap<String, HashMap<String, Vec<SolcItem>>>,
}

/// Optimizer Settings
#[derive(Serialize, Debug, Clone)]
struct Optimizer {
    /// disabled by default
    enabled: true,
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
#[derive(Serialize, Debug, Clone)]
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
    Bytecode(EvmOpt),
    DeployedBytecode(EvmOpt),
    /// eWASM s-expressions format (not currently supported)
    EwasmWast,
    /// eWASM binary format (not currently supported)
    EwasmWasm,

}

EvmOpt {
    /// New Assembly Format after Desugaring
    Assembly,
    /// Old-style assembly format in JSON
    LegacyAssembly,
    /// Bytecode Object
    BytecodeObject,
    /// Opcodes List
    Opcodes,
    /// Source Mapping
    SourceMap,
    /// Link References (if unlinked object)
    LinkReferences,
    /// The list of function hashses
    MethodIdentifiers,
    /// Function gas estimates
    GasEstimates
}

#[cfg(test)]
mod tests {
    use super::*;

}
