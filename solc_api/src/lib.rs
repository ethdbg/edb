//! Standard JSON Input/Output for the Solidity Compiler
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate solc;
extern crate ethereum_types;
extern crate ethabi;
extern crate failure;
extern crate hex;

mod internal_types;
mod err;
pub use self::err::SolcApiError;
pub use self::internal_types::output::{CompiledSource, Contract};

pub mod types {
    pub use super::internal_types::output::{
        LegacyAst, Metadata, UserDoc, DevDoc, LegacyAssembly, MethodIdentifiers,
        Ast, EWasm, GasEstimates, CreationGasEstimates, Evm, Bytecode, Instruction,
        SourceIndex, Jump, Position
    };
    pub use super::internal_types::input::{FoundationVersion};
    pub use ethabi::Contract as ContractAbi;
}

use self::internal_types::input::*;

use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct SolcApiBuilder {
    /// specify the path of the source code
    source: PathBuf,
    /// EvmVersion to use
    version: Option<String>,
    /// [FLAG] whether to optimize output
    optimize: Option<bool>,
}

impl SolcApiBuilder {
    // fs::canonicalize
    pub fn source_file(&mut self, val: PathBuf) -> &mut Self {
        let new = self;
        new.source = val;
        new
    }
    /// Version
    pub fn evm_version(&mut self, ver: FoundationVersion) -> &mut Self {
        let new = self;
        new.version = Some(ver.into());
        new
    }
    /// Whether to optimize output
    pub fn optimize(&mut self) -> &mut Self {
        let new = self;
        new.optimize = Some(true);
        new
    }

    /// returns Standard JSON input for Solidity Compiler
    // TODO: Return errors and do not panic
    // TODO: Make work for multiple input files
    pub fn build(&self) -> String {
        let mut default = StandardJson::default();
        let source_path = self.source
            .clone()
            .canonicalize()
            .expect("Could not get absoulte path of Source File");
        if let Some(name) = self.source.file_name() {
            default.sources.insert(name.to_str().expect("File Name is not valid UTF-8!").to_string(), SourceFile {
                urls: Some(vec![source_path]),
                content: None,
                hash: None,
            });
        } else {
            panic!("Path does not terminate in File Name");
        }
        serde_json::to_string(&default).expect("Could not build Standard JSON Object")
    }

    pub fn compile(&self) -> CompiledSource {
        let json = self.build();
        if let Some(p) = self.source.canonicalize().unwrap().parent() {
            let compiled = solc::standard_json(&json, Some(vec![p])).expect("Compilation Failed");
            let raw = serde_json::from_str(&compiled).expect("Deserializing standard json output failed");
            CompiledSource::new(raw)
        } else {
            let compiled = solc::standard_json(&json, None).expect("Compilation Failed");
            let raw = serde_json::from_str(&compiled).expect("Deserializing standard json output failed");
            CompiledSource::new(raw)
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ser_opts() {
        let solc_items = vec![SolcItem::Abi,
                              SolcItem::Bytecode(EvmOpt::BytecodeObject),
                              SolcItem::DeployedBytecode(EvmOpt::BytecodeObject)
        ];
        let ser_items = serde_json::to_string(&solc_items).unwrap();
        assert_eq!(ser_items, r#"["abi","evm.bytecode.object","evm.deployedBytecode.object"]"#);
    }

    #[test]
    fn ser_compilation_object() {
        let obj = StandardJson::default();
        let ser = serde_json::to_string(&obj).unwrap();
        assert_eq!(ser, r#"{"language":"Solidity","sources":{},"settings":{"outputSelection":{"*":{"*":["abi","ast","evm.deployedBytecode.object","evm.deployedBytecode.sourceMap"]}}}}"#);
    }

    #[test]
    fn build_standard_json() {
        let _json = SolcApiBuilder::default()
            .source_file(PathBuf::from("./../tests/contracts/solidity/voting/voting.sol"))
            .build();
    }

    #[test]
    fn compile_standard_json() {
        let _compiled = SolcApiBuilder::default()
            .source_file(PathBuf::from("./../tests/contracts/solidity/voting/voting.sol"))
            .compile();
    }
}
