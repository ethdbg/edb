//! Mock web3 functions
//! should only be used for tests
//! Very simple, can be expanded to support parameters for methods
//! Only contains getTransaction, getBalance, getCode, and getStorage methods
//! if address 0x884531eab1ba4a81e9445c2d7b64e29c2f14587c is passed in for 'getCode', the code for
//! the solidity/simple.bin/SimpleStorage.bin is returned.
//! all other values are 0/uninitialized values
//! a balance of 150,000,000 wei is used for getBalance

use web3::{Transport, RequestId, helpers::CallFuture };
use futures::future::Future;
use jsonrpc_core::{Call, MethodCall, Version, Id, Params};
use serde_json::{json, value::Value};
use log::{info, log};

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

    fn send(&self, id: RequestId, call: Call) -> Self::Out {
        Box::new(futures::future::result(Ok(self.send_request(call))))
    }
}

/// Returns one constant for every method
impl MockWeb3Transport {
    fn send_request(&self, call: Call) -> Value {
        let method = match call {
            Call::MethodCall(method) => method,
            _ => panic!("Only method calls supported for mockweb3")
        };
        let mut addr: Option<String> = None;
        if let Some(x) = method.params.clone() {
            let params: Value = x.parse().unwrap();
            addr = Some(serde_json::from_value(params.get(0).unwrap().clone()).unwrap());
        }

        // let val: Value = serde_json::from_str()
        info!("Method: {:?}", method);
        let val: Value = match method.method.as_ref() {
            "eth_getTransactionCount" => {
                serde_json::from_str(r#""0x0""#).unwrap()
            },
            "eth_getBalance" => {
                serde_json::from_str(r#""0x8F0D180""#) .unwrap()
            },
            "eth_getCode" => {
                if addr.is_some() && 
                    addr.expect("scope is conditional; qed") == "0x884531eab1ba4a81e9445c2d7b64e29c2f14587c" 
                {
                    let jstr = json!(include!("solidity/simple.bin/SimpleStorage.bin-runtime"));
                    jstr
                } else {
                    serde_json::from_str(r#""0x601714""#).unwrap()
                }
            },
            "eth_getStorageAt" => {
                serde_json::from_str(r#""0x0000000000000000000000000000000000000000000000000000000000000000""#).unwrap()
            }
            _ => panic!("method not found")
        };
        val
    }
}
