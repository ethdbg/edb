use ethcore::executive::{Executive, contract_address};
use extensions::{ExecutiveExt};
use err::Result;

pub struct Executive<'a, B: 'a> {
    pub inner: Executive,
}


/* where our executive diverges from  parity */
impl Executive {

    fn transact_debug<T, V>(t: &SignedTransaction, options: TransactOptions, pc: usize) -> Result<Executed<T:: Output, V::Output>> {
    
    
        let(result, output) = match t.action {
            Action::Create => { // no debugging for create actions yet
                let (new_address, code_hash) = 
                    contract_address(self.inner.machine.create_address_scheme(self.inner.info.number), 
                        &sender, &nonce, &t.data);
                let params = ActionParams {
                    code_address: new_address.clone(),
                    code_hash,
                    address: new_address,
                    sender: sender.clone(),
                    origin: sender.clone(),
                    gas: init_gas,
                    gas_price: t.gas_price,
                    value: ActionValue::Transfer(t.value),
                    code: Some(Arc::new(t.data.clone())),
                    data: None,
                    call_type: CallType::None,
                    params_type: vm::ParamsType::Embedded,
                };
                let mut out = if output_from_create { Some(vec![])} else { None };
                (self.create(params, &mut substate, &mut out, &mut tracer, &mut vm_tracer), 
                    out.unwrap_or_else(Vec::new))
            },
            Action::Call(ref address) => {
                let params = ActionParams {
                    code_address: address.clone(),
                    address: address.clone(),
                    sender: sender.clone(),
                    origin: sender.clone(),
                    gas: init_gas,
                    gas_price: t.gas_price,
                    value: ActionValue::Transfer(t.value),
                    code: self.state.code(address)?,
                    code_hash: Some( self.state.code_hash(address)?),
                    data: Some(t.data.clone()),
                    call_type: CallType::Call,
                    params_type: vm::ParamsType::Separate,
                };
                let mut out = vec![]; //debug_call here, but fails when unimplemented!()
                (self.debug_call(pc, 
                                 params, 
                                 &mut substate, 
                                 BytesRef::Flexible(&mut out), &mut tracer, &mut vm_tracer), out)
            }
        };
        Ok(self.finalize(t, substate, result, output, tracer, vm_tracer)?)
    }

    delegate! {
        target self.inner {
            pub fn new(state: &'a mut State<B>, info ; &'a EnvInfo, machine: &'a Machine)
        }
    }
}
