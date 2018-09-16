//! Standard JSON Input/Output for the Solidity Compiler
mod input;
mod output;
use self::{
    input::*,
    output::*,
};
use std::path::PathBuf;
use crate::types::{FoundationVersion};

#[derive(Debug, Clone, Default)]
pub struct StandardJsonBuilder {
    /// specify the path of the source code
    source: PathBuf,
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
    /// Version
    fn evm_version(&mut self, ver: FoundationVersion) -> &mut Self {
        let new = self;
        new.version = Some(ver.into());
        new
    }
    /// Whether to optimize output
    fn optimize(&mut self) -> &mut Self {
        let new = self;
        new.optimize = Some(true);
        new
    }

    /// returns Standard JSON input for Solidity Compiler
    //TODO: Return errors and do not panic
    // TODO: Make work for multiple input files
    fn build(&self) -> String {
        let mut default = StandardJson::default();
        let source_path = self.source
            .clone()
            .canonicalize()
            .expect("Could not get absoulte path of Source File");
        let mut map = None;
        if let Some(rem) = &source_path.parent() {
            map = Some(String::from("=") + rem.to_str().expect("Path is not Valid UTF-8"))
        }
        let url = url::Url::from_file_path(source_path.as_path())
            .expect("Could not convert path to URL");

        if let Some(name) = self.source.file_name() {
            default.sources.insert(name.to_str().expect("File Name is not valid UTF-8!").to_string(), SourceFile {
                urls: Some(vec![UrlType(url)]),
                content: None,
                hash: None,
            });
        } else {
            panic!("Path does not terminate in File Name");
        }
        if map.is_some() {
            default.settings.remappings = Some(vec![map.expect("Scope is conditional")]);
        }
        serde_json::to_string(&default).expect("Could not build Standard JSON Object")
    }

    fn compile(&self) -> CompiledSource {
        let json = self.build();
        serde_json::from_str(&json).expect("Compilation Failed")
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
        let json = StandardJsonBuilder::default()
            .source_file(PathBuf::from("./../tests/contracts/solidity/voting/voting.sol"))
            .build();
        println!("JSON: {}", json);
    }

    #[test]
    fn compile_standard_json() {
        let compiled = StandardJsonBuilder::default()
            .source_file(PathBuf::from("./../tests/contracts/solidity/voting/voting.sol"))
            .compile();
        println!("{:?}", compiled);
    }
}
