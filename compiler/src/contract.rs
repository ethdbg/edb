//! Contract Interface for Codefile/SourceMap/Debugger operations
use super::{err::{LanguageError, NotFoundError}, Ast, SourceMap, AbstractFunction, AstItem, CharOffset};


use ethereum_types::Address;
use delegate::*;
use std::{path::PathBuf, rc::Rc};
use failure::Error;
use log::*;

pub struct ContractFile {
    /// Identifier for source file (used in Source Maps)
    id: usize,
    /// path to source file
    file_path: PathBuf,
    /// name of source file
    file_name: String,
    source: String,
    /// General source map for offsets--line number
    // Abstract Syntax Tree of Source
    ast: Box<dyn Ast>,
}

impl ContractFile {
    pub fn new(source: String, id: usize, ast: Box<dyn Ast>, file_path: PathBuf)
        -> Result<Self, Error>
    {
        let file_name = file_path
            .file_name()
            .ok_or(LanguageError::NotFound(NotFoundError::File))?
            .to_str()
            .ok_or(LanguageError::InvalidPath)?
            .to_string();

        Ok(Self {
            source, file_name, file_path, id, ast
        })
    }

    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    delegate! {
        target self.ast {
            pub fn variable(&self, name: &str) -> Result<AstItem, Error>;
            pub fn contract(&self, name: &str) -> Result<AstItem, Error>;
            pub fn function(&self, name: &str, fun: &mut FnMut(Result<&AbstractFunction, Error>) -> bool) -> Result<AstItem, Error>;
            pub fn find_contract(&self, offset: CharOffset) -> Option<AstItem>;
            pub fn find_function(&self, fun: &mut FnMut(&AbstractFunction) -> bool) -> Option<AstItem>;
        }
    }
}

// get rid of Web3 here. Only thing contract is needed for is querying local functions + AST. We
// never really need to query the contract functions.
/// Contract
pub struct Contract {
    file: Rc<ContractFile>,
    name: String,
    abi: ethabi::Contract,
    runtime_bytecode: Vec<u8>,
    source_map: Box<dyn SourceMap>,
    addr: Address,
}

// contract interface for the debugger. should be the same across all languages
impl Contract {

    delegate! {
        target self.abi {
            /// Creates abi constructor call builder
            pub fn constructor(&self) -> Option<&ethabi::Constructor>;
            /// Creates abi function call builder
            pub fn function(&self, name: &str) -> Result<&ethabi::Function, ethabi::Error>;
            /// Creates abi event decoder
            pub fn event(&self, name: &str) -> Result<&ethabi::Event, ethabi::Error>;
            /// Iterate over all functions of the contract in arbitrary order
            pub fn functions(&self) -> ethabi::Functions;
            /// iterate over all events of the contract abi in arbitrary order
            pub fn events(&self) -> ethabi::Events;
            /// Returns true if contract abi has fallback
            pub fn fallback(&self) -> bool;
        }
    }

    pub fn new(file: Rc<ContractFile>,
               name: String,
               map: Box<dyn SourceMap>,
               abi: ethabi::Contract,
               addr: &Address,
               runtime_bytecode: Vec<u8>) -> Result<Self, Error>
    {
        trace!("Contract Instantiation Code Length: {}", runtime_bytecode.len());
        trace!("{:?}", runtime_bytecode);
        Ok(Self { addr: addr.clone(), file, name, abi, runtime_bytecode, source_map: map })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn file(&self) -> Rc<ContractFile> {
        self.file.clone()
    }

    /// Returns address on testnet that the contract is deployed at
    pub fn address(&self) -> Address {
        self.addr
    }

    /// get the source map of this contract
    pub fn source_map(&self) -> &Box<dyn SourceMap> {
        &self.source_map
    }
}
