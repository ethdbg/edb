//! Mock web3 functions
//! should only be used for tests
//! Very simple, can be expanded to support parameters for methods
//! Only contains getTransaction, getBalance, getCode, and getStorage methods
//! if address 0x884531eab1ba4a81e9445c2d7b64e29c2f14587c is passed in for 'getCode', the code for
//! the solidity/simple.bin/SimpleStorage.bin is returned.
//! all other values are 0/uninitialized values
//! a balance of 150,000,000 wei is used for getBalance

use web3::{Transport, RequestId};
use futures::future::Future;
use jsonrpc_core::{Call, MethodCall, Version, Id, Params};
use serde_json::{json, value::Value};
use log::*;
use super::types::*;

#[derive(Clone, Debug, Default)]
pub struct MockWeb3Transport;


impl Transport for MockWeb3Transport {
    type Out=Box<Future<Item=Value, Error=web3::error::Error>>;
    /// A mock call will always be a method call with ID 0
    fn prepare(&self, method: &str, params: Vec<Value>)  -> (RequestId, Call) {
        (0, Call::MethodCall(MethodCall{
            jsonrpc: Some(Version::V2),
            method: method.to_string(),
            params: Some(Params::Array(params)),
            id: Id::Num(0)}))
    }

    fn send(&self, _id: RequestId, call: Call) -> Self::Out {
        Box::new(futures::future::result(Ok(self.send_request(call))))
    }
}

/// Returns one constant for every method
impl MockWeb3Transport {
    fn send_request(&self, call: Call) -> Value {
        info!("Mock call initiated: {:?}", call);
        let method = match call {
            Call::MethodCall(method) => method,
            _ => panic!("Only method calls supported for mockweb3")
        };
        let mut addr: Option<String> = None;
        if let Some(x) = method.params.clone() {
            info!("method.params: {:?}", method.params);
            let params: Value = x.parse().expect("Mock parse failure");
            if params.get(0).is_none() {
                addr = None;
            } else {
                addr = Some(serde_json::from_value(params.get(0).expect("conditional scope").clone()).expect("Mock failure; deserialize"));
            }
        }

        // let val: Value = serde_json::from_str()
        info!("Method: {:?}", method);
        let val: Value = match method.method.as_ref() {
            "eth_getTransactionCount" => {
                info!("Returning a transaction count of 0");
                serde_json::from_str(r#""0x0""#).expect("Could not decode tx count json")
            },
            "eth_getBalance" => {
                info!("Returning a balance of 150,000,000 WEI");
                serde_json::from_str(r#""0x8F0D180""#) .expect("Could not decode balance json")
            },
            "eth_getCode" => {
                if addr.is_none() {
                    warn!("No address supplied in parameters to mock RPC!");
                    return serde_json::from_str(r#""0x601714""#).expect("Could not decode empty code json");
                }
                let address = addr.expect("No address supplied in parameters to mock RPC!");
                if address == SIMPLE_STORAGE_ADDR {
                    json!(SIMPLE_STORAGE_BYTECODE_RUNTIME)
                } else if address == VOTING_ADDR {
                    json!(VOTING_BYTECODE_RUNTIME)
                } else { // if no addresses match, return 'empty' code
                    warn!("Could not find a matching address, returning no code");
                    serde_json::from_str(r#""0x601714""#).expect("Could not decode empty code json")
                }
            },
            "eth_getStorageAt" => {
                warn!("Mock RPC does not keep real storage, returning arbitrary storage value");
                serde_json::from_str(r#""0xffffff0000000000000000000000000000000000000000000000000000000000""#)
                    .expect("Could not decode arbitrary storage json")
            },
            "eth_accounts" => {
                let json = json!([
                    SIMPLE_STORAGE_ADDR,
                    VOTING_ADDR,
                ]);
                info!("`eth_accounts`: {:?}", json);
                json
            }
            _ => panic!("method not found")
        };
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::Future;

    #[test]
    fn accounts() {
        pretty_env_logger::try_init();
        let mock = MockWeb3Transport::default();
        let client = web3::Web3::new(mock);
        let acc = client.eth().accounts().wait().unwrap();
        info!("Accounts: {:?}", acc);
    }

    #[test]
    fn get_code() {
        pretty_env_logger::try_init();
        let mock = MockWeb3Transport::default();
        let client = web3::Web3::new(mock);
        let code = client.eth().code(ethtype_addr(VOTING_ADDR), None).wait().unwrap();
    }
}
