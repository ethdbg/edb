
use jsonrpc_core::{MetaIoHandler, Params, Value};
use edb_core::{Debugger, Language, Transport, CompiledFiles};

pub fn methods<T>(io: &mut MetaIoHandler, dbg: &mut Debugger<T>) where T: Transport {

    io.add_method("say_hello", |_params: Params| {
        Ok(Value::String("hello".to_string()))
    });
}
