
mod methods;

use super::lib::File;

use log::*;
use failure::Error;
use jsonrpc_core::{IoHandler};
use jsonrpc_minihttp_server::{ServerBuilder};
use ethereum_types::Address;
use edb_core::{Debugger, Language, Transport, CompiledFiles};

pub struct Rpc<T> where T: Transport {
    dbg: Option<Debugger<T>>,
    files: CompiledFiles, // TODO combine files with File struct
    client: web3::Web3<T>,
    addr: Address,
    root_file: File,
    current: Option<Vec<String>>
}

impl<T> Rpc where T: Transport {
    pub fn new<L>(lang: L, client: web3::Web3<T>, addr: Address, file: File) -> Result<Self, Error> where L: Language {
        debug!("File: {:?}", file);
        Ok(Self {
            dbg: None,
            files: file.compile(lang, &addr)?,
            client, 
            addr, 
            root_file: file,
            current: None
        })
    }

    pub fn run<L, T>(&mut self) where L: Language, T: Transport {
        let mut io = IoHandler::new();
        methods::methods(&mut io, &mut self.dbg); 
        let server = ServerBuilder::new(io)
            .threads(3)
            .start_http(&"127.0.0.1:3030".parse().unwrap())
            .unwrap();
        server.wait().unwrap();
    }
}

