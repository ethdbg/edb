//! Mock web3 functions
//! should only be used for tests
//! Very simple, can be expanded to support parameters for methods

use web3::{Transport, RequestId, helpers::CallFuture };
use futures::future::Future;
use jsonrpc_core::{Call, MethodCall, Version, Id, Params};
use serde_json::value::Value;
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

        // let val: Value = serde_json::from_str()

        let val: Value = match method.method.as_ref() {
            "SomeMethod" => {
                serde_json::from_str(r#"{"A CONST"}"#).unwrap()
            },
            _ => panic!("No method found")
        };

        info!("METHOD: {:?}", method);
        panic!("Should not have to send any requests in tests");
        return serde_json::from_str(r#"{"A COSNT"}"# ).unwrap();
    }
}
