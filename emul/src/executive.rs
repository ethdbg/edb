use {crossbeam, evm, err, std, vm};
use ethcore::executive::{
    Executive as ParityExecutive, contract_address, TransactOptions,
    STACK_SIZE_PER_DEPTH, STACK_SIZE_ENTRY_OVERHEAD
};
use ethcore::executed::{Executed, ExecutionError};
use ethcore::externalities::*;
use ethcore::state::{Backend as StateBackend, Substate, CleanupMode};
use ethcore::trace::{Tracer, VMTracer, FlatTrace, VMTrace};
use ethcore_io::LOCAL_STACK_SIZE;
use vm::{self, Schedule, ActionParams, ActionValue, CleanDustMode, Ext, ReturnData, GasLeft};
use evm::{Finalize, CallType};
use ethereum_types::{U256, U512};
use bytes::{Bytes, BytesRef};
use transaction::{Action as TxAction, SignedTransaction};
use std::sync::Arc;
use raton;

use emulator::{Action, Emulator, VMEmulator, EDBFinalize, FinalizationResult};
use externalities::{DebugExt, ExternalitiesExt};
use extensions::{ExecInfo, FactoryExt};

pub enum ExecutionState {
    Executing,
    Done(vm::Result<GasLeft>),
}

pub struct Executive<'a, B: 'a> {
    pub inner: Executive<'a, B>,
    pool: Option<rayon::ThreadPool>, // a pool so we don't have to keep creating/uninstalling threads whenever debug_resume is called
        // rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
}

/* where our executive diverges from  parity */
impl<'a, B: 'a + StateBackend> Executive<'a, B> {
    pub fn new(state: &'a mut State<B>, info ; &'a EnvInfo, machine: &'a Machine) -> Result<Self> {
    
        Ok(Executive {
            inner: ParityExecutive::new(state, info, machine),
            pool: None,
       })
    }

    fn transact_debug<T, V>(&mut self, t: &SignedTransaction, 
                            options: TransactOptions) -> err::Result<Executed<T:: Output, V::Output>> {
        let output_from_create = options.output_from_init_contract;
        let(sender, init_gas, substate, 
            tracer, vm_tracer, substate) = self._transact_debug(t, options)?;
        let nonce = self.state.nonce(&sender)?;

        let(result, output) = match t.action {
            TxAction::Create => { // no debugging for create actions yet
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
            TxAction::Call(ref address) => {
                let params = ActionParams {
                    code_address: address.clone(),
                    address: address.clone(),
                    sender: sender.clone(),
                    origin: sender.clone(),
                    gas: init_gas,
                    gas_price: t.gas_price,
                    value: ActionValue::Transfer(t.value),
                    code: self.inner.state.code(address)?,
                    code_hash: Some( self.inner.state.code_hash(address)?),
                    data: Some(t.data.clone()),
                    call_type: CallType::Call,
                    params_type: vm::ParamsType::Separate,
                };
                let mut out = vec![];
                (self.debug_call(params, &mut substate, 
                                 BytesRef::Flexible(&mut out), &mut tracer, &mut vm_tracer), out)
            }
        };
        Ok(self.finalize(t, substate, result, output, tracer, vm_tracer)?)
    }
       
    fn debug_call<T,V>(&mut self, params: ActionParams, substate: &mut Substate, output: BytesRef,
                       tracer: &mut T, 
                       vm_tracer: &mut V
    ) -> Result<evm::FinalizationResult> where T: Tracer, V: VMTracer {
        if let Some(schedule, params, unconfirmed_substate, output_policy, tracer, vm_tracer) 
                = self._debug_call(params, substate, output, tracer, vm_tracer) 
        {
            let (ext, vm) = self.init_vm(schedule, 
                                         params, 
                                         unconfirmed_substate, 
                                         output_policy, 
                                         tracer, 
                                         vm_tracer);
            // poll for events
            // execute debug_resume() on events
            //
        } else {
            self.state.discard_checkpoint();
            tracer.trace_call(trace_info, U256::zero(), trace_output, vec![]);
            Ok(evm::FinalizationResult {
                gas_left: params.gas,
                return_data: ReturnData::empty(),
                apply_state: true
            })
        }
    }

    fn init_vm(&mut self, 
               schedule: Schedule, 
               params: ActionParams, 
               unconfirmed_substate: &mut Substate,
               output_policy: OutputPolicy,
               tracer: &mut T,
               vm_tracer: &mut V
    ) -> Result<(ext, vm)> where T: Tracer, V: VMTracer {
        let local_stack_size = LOCAL_STACK_SIZE.with(|sz| sz.get());
        let depth_threshold = local_stack_size.saturating_sub(STACK_SIZE_ENTRY_OVERHEAD) / STACK_SIZE_PER_DEPTH;
        let static_call = params.call_type == CallType::StaticCall;
        
        // Ordinary execution - keep VM in same thread
        let vm_factory = self.state.vm_factory();
        let mut ext = self.as_dbg_externalities(OriginInfo::from(&params), unconfirmed_substate, 
                                                output_policy, tracer, vm_tracer, static_call);
        trace!(target: "executive", "ext.schedule.have_delegate_call: {}", 
               ext.schedule().have_delegate_call);
        let vm: Box<VMEmulator> = vm_factory.create_debug(params, &ext);
        
        let self.pool 
            = self.pool::new()
                    .num_threads(2)
                    .stack_size(std::cmp::max(schedule.max_depth.saturating_sub(depth_threshold) * STACK_SIZE_PER_DEPTH, local_stack_size))
                    .build()?;
        Ok(ext, vm)
    }
    
    fn debug_resume(action: Action, ext: impl ExternalitiesExt, vm: Box<VMEmulator>
    ) -> Result<ExecInfo> { // should return ExecInfo
        // this will run in a threadpool
        self.pool.install(move | | vm.fire(action, &mut ext))?;
    }


    delegate! {
        target self.inner {
            // pub fn new(state: &'a mut State<B>, info ; &'a EnvInfo, machine: &'a Machine) -> Self;

            pub fn finalize<T,V>(&mut self, 
                                 t: &SignedTransaction, 
                                 substate: Substate, 
                                 result: vm::Result<FinalizationResult>, 
                                 output: Bytes, 
                                 trace: Vec<T>, 
                                 vm_trace: Option<V>
            ) -> Result<Executed<T, V>, ExecutionError>;
        }
    }
}

impl ExecutiveExt for Executive {
    delegate! {
        target self.inner {
            fn _transact_debug<T, V>(&'a mut self, 
                                    t: &SignedTransaction,
                                    options: TransactOptions<T,V>,
            ) -> Result<(sender: Address, init_gas: U256, tracer: T, vm_tracer: V, substate: &mut Substate)> 
                where T: Tracer,
                      V: VMTracer;
             fn _debug_call<T,V>( &mut self, params: ActionParams,
                                  substate: &mut Substate,
                                  output: BytesRef,
                                  tracer: &mut T,
                                  vm_tracer: &mut V
            ) -> Result<evm::FinalizationResult> where T: Tracer, V: VMTracer;
    
        }
    }
}
