mod conf;
mod shell;

use edb_core::Transport;
use self::conf::{FileType, Mode};
use self::shell::Shell;

use failure::Error;
use log::*;
// Get user input from Config
//  - (File Type)
//  - RPC


fn main() -> Result<(), Error> {
    let conf = conf::Configuration::new()?;
    // TODO: Clean this up
    
    // Take care of the 'Transport' Generic based on CLI Arguments
    match *conf.file.file_type() {
        FileType::Solidity => {
            let trans = conf.transport().clone();
            match trans.scheme_part().map(|s| s.as_str()) {
                Some("http") | Some("https") => {
                    let http = web3::transports::Http::new(into_str(trans).as_str())
                        .unwrap_or_else(|e| {
                            error!("{}", e);
                            std::process::exit(1);
                        });
                    let client = web3::Web3::new(http);
                    start_provider(conf, client);
                },
                Some("file") => {
                    let ipc = web3::transports::Ipc::new(into_str(trans).as_str())
                        .unwrap_or_else(|e| {
                            error!("{}", e);
                            std::process::exit(1);
                        }); 
                    let client = web3::Web3::new(ipc);
                    start_provider(conf, client);
                },
                Some("ws") => {
                    let ws = web3::transports::WebSocket::new(into_str(trans).as_str())
                        .unwrap_or_else(|e| {
                            error!("{}", e);
                            std::process::exit(1);
                        });
                    let client = web3::Web3::new(ws);
                    start_provider(conf, client);
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

fn start_provider<T>(conf: conf::Configuration, client: web3::Web3<T>) 
    -> Result<(), Error> where T: Transport
{
    match *conf.mode() {
        Mode::Tui => Shell::new().run()?,
        Mode::Rpc => unimplemented!(),
    }
    Ok(())
}

fn into_str(uri: http::uri::Uri) -> String {
    let mut transport = String::new();
    let parts = uri.into_parts();
    if let Some(scheme) = parts.scheme {
        transport.extend(scheme.as_str().chars());
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
