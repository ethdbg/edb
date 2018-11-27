mod conf;
mod shell;
mod lib;
mod err;
// mod rpc;

use edb_core::{Transport, Language, Solidity};
use self::{
    conf::Mode,
    shell::Shell,
    lib::FileType
};

use failure::Error;
use log::*;


// Get user input from Config
//  - (File Type)
//  - RPC
//TODO use LanguageType enum from compiler crate

fn main() -> Result<(), Error> {
    let conf = conf::Configuration::new()?;
    // TODO: Clean this up
    // could probably use some kind of Iterator implementation on a 'Languages' enum, or 'Transports' enum, or
    // something similar to that instead of manually matching everything (lots of repeated code)
    // in addition, the errors should be propogated in order to avoid lots of error-handling
    // boilerplate

    // Take care of the 'Transport' Generic based on CLI Arguments
    match *conf.file.file_type() {
        FileType::Solidity => {
            let trans = conf.transport().clone();
            debug!("Transport {}", into_str(trans.clone()));
            match trans.scheme_part().map(|s| s.as_str()) {
                Some("http") | Some("https") => {
                    let (_eloop, http) = web3::transports::Http::new(into_str(trans).as_str())
                        .unwrap_or_else(|e| {
                            error!("{}", e);
                            std::process::exit(1);
                        });
                    let client = web3::Web3::new(http);
                    start_provider(conf, client, Solidity::default())?;
                },
                Some("file") => {
                    // TODO: This probably won't work. IPC transport expects a normal filepath
                    let (_eloop, ipc) = web3::transports::Ipc::new(into_str(trans).as_str()) 
                        .unwrap_or_else(|e| {
                            error!("{}", e);
                            std::process::exit(1);
                        });
                    let client = web3::Web3::new(ipc);
                    start_provider(conf, client, Solidity::default())?;
                },
                Some("ws") => {
                    let (_eloop, ws) = web3::transports::WebSocket::new(into_str(trans).as_str())
                        .unwrap_or_else(|e| {
                            error!("{}", e);
                            std::process::exit(1);
                        });
                    let client = web3::Web3::new(ws);
                    start_provider(conf, client, Solidity::default())?;
                },
                None => {
                    error!("Must provide scheme of URI for eth RPC");
                    std::process::exit(1);
                },
                _ => {
                    error!("Invalid Scheme Provided");
                    std::process::exit(1);
                }
            }
        },
        _ => {
            error!("Language not supported");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn start_provider<T>(conf: conf::Configuration, client: web3::Web3<T>, lang: impl Language)
    -> Result<(), Error> where T: Transport
{
    match *conf.mode() {
        Mode::Tui => Shell::<T>::new(lang, client, conf.addr().clone(), conf.file().clone())?.run()?,
        Mode::Rpc => unimplemented!(),
    }
    Ok(())
}

fn into_str(uri: http::uri::Uri) -> String {
    let mut transport = String::new();
    let parts = uri.into_parts();
    if let Some(scheme) = parts.scheme {
        transport.extend(scheme.as_str().chars());
        transport.extend("://".chars());
    } else {
        error!("Must specify a uri scheme"); // TODO: propogate up
        std::process::exit(1);
    }
    if let Some(authority) = parts.authority {
        transport.extend(authority.as_str().chars());
    }
    if let Some(p_and_q) = parts.path_and_query {
        transport.extend(p_and_q.as_str().chars());
    }
    transport
}
