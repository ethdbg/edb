use {err, evm, std, vm, rayon};
use ethcore::executive::{
    Executive as ParityExecutive, contract_address, TransactOptions,
    STACK_SIZE_PER_DEPTH, STACK_SIZE_ENTRY_OVERHEAD
};
use ethcore::executed::{Executed, ExecutionError};
use ethcore::externalities::*;
use ethcore::state::{Backend as StateBackend, Substate, State};
use ethcore::trace::{Tracer, VMTracer};
use ethcore::trace::trace::Call;
use ethcore::machine::EthereumMachine as Machine;
use ethcore::factory::VmFactory;
use ethcore_io::LOCAL_STACK_SIZE;
use vm::{Schedule, ActionParams, ActionValue, Ext, ReturnData, GasLeft, EnvInfo};
use evm::{Finalize, CallType};
use ethereum_types::{U256, U512, Address};
use bytes::{Bytes, BytesRef};
use transaction::{Action as TxAction, SignedTransaction};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::sync::mpsc;

use emulator::{Action, VMEmulator, EDBFinalize, FinalizationResult};
use externalities::{DebugExt, ExternalitiesExt};
use extensions::{ExecInfo, FactoryExt, ExecutiveExt};
use err::{Error, InternalError, DebugError};
use utils::DebugReturn;

pub enum ExecutionState {
    Executing,
    Done(vm::Result<GasLeft>),
}

pub struct Executive<'a, B: 'a> {
    pub inner: ParityExecutive<'a, B>,
    pool: Option<rayon::ThreadPool>, // a pool so we don't have to keep creating/uninstalling threads whenever debug_resume is called
        // rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
}

/* where our executive diverges from  parity */
impl<'a, B: 'a + StateBackend> Executive<'a, B> {
    pub fn new(state: &'a mut State<B>, info: &'a EnvInfo, machine: &'a Machine) -> err::Result<Self> {
    
        Ok(Executive {
            inner: ParityExecutive::new(state, info, machine),
            pool: None,
       })
    }

    fn transact_debug<T: 'a, V: 'a>(&mut self, 
                            t: &SignedTransaction, 
                            options: TransactOptions<T, V>,
                            rx: mpsc::Receiver<Action>,
                            tx: mpsc::Sender<ExecInfo>
    ) -> err::Result<Executed<T:: Output, V::Output>> where T: Tracer, V: VMTracer
    {
        let output_from_create = options.output_from_init_contract;
        let mut tracer = options.tracer;
        let mut vm_tracer = options.vm_tracer;
        let mut substate = Substate::new();
      
        // basically just performs checks
        let (sender, init_gas, gas_cost) = self._transact_debug(t, options.check_nonce)?;
        

        let schedule = self.inner.machine.schedule(self.inner.info.number);

        if !schedule.eip86 || !t.is_unsigned() {
            self.inner.state.inc_nonce(&sender)?;
        }
    
        self.inner.state.sub_balance(&sender, 
                               &U256::from(gas_cost), 
                               &mut substate.to_cleanup_mode(&schedule))?;

        let nonce = self.inner.state.nonce(&sender)?;

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
                (Ok(self.debug_call(params, &mut substate, 
                                 BytesRef::Flexible(&mut out), &mut tracer, &mut vm_tracer, rx, tx)?), out)
            }
        };
        Ok(self.finalize(t, substate, result, output, tracer.drain(), vm_tracer.drain())?)
    }
       
    fn debug_call<T: 'a, V: 'a>(&mut self, params: ActionParams, substate: &mut Substate, output: BytesRef,
                       tracer: &mut T, 
                       vm_tracer: &mut V,
                       rx: mpsc::Receiver<Action>,
                       tx: mpsc::Sender<ExecInfo>,
    ) -> vm::Result<evm::FinalizationResult> where T: Tracer, V: VMTracer {
        let mut ret = self._debug_call(&params, substate, tracer, vm_tracer)?;
        
        /* unwraps will never panic because is_code indicates whether values are of Some or None
         * type in `ret` struct */
        if ret.is_code() {
            let static_call = params.call_type == CallType::StaticCall;
            let mut uncon_sub = ret.unconfirmed_substate.unwrap();
            let res = { 
                let mut info: Option<ExecInfo> = None;
                let vm_factory = self.inner.state.vm_factory();
                let output_policy = OutputPolicy::Return(output, ret.trace_output.as_mut());
                let mut ext = self.as_dbg_externalities(OriginInfo::from(&params), 
                                                        &mut uncon_sub,
                                                        output_policy, tracer, vm_tracer, 
                                                        static_call);

                let (mut vm, pool) = Self::init_vm(&ext, ret.schedule.unwrap(), params.clone(), vm_factory)?;
                let mut vm: Arc<VMEmulator + Send + Sync> = Arc::from(vm);
                // let mut vm: Arc<Mutex<RefCell<Box<VMEmulator + Send + Sync>>>> = Arc::new(Mutex::new(RefCell::new(vm)));

                for x in rx.iter() {
                    info = Some(Self::debug_resume(x, &mut ext, &mut vm.clone(), &pool)?);
                    tx.send(info.clone().expect("Info was just put into `Some`; qed"));
                }
                let info = info.ok_or_else(|| vm::Error::Internal("Execution Info returned as `None` Value".to_owned()))?;

                match info.gas_left() {
                    Some(gas) => Ok(gas).finalize(ext),
                    None => Err(vm::Error::Internal("Execution failed because gas has value `None`".to_string())).finalize(ext),
                }
            };
        
            vm_tracer.done_subtrace(ret.subvmtracer.unwrap());
            trace!(target: "executive", "res={:?}", res);
            let traces = ret.subtracer.unwrap().drain();
            
            match res {
                Ok(ref res) if res.apply_state => tracer.trace_call(ret.trace_info, params.gas - res.gas_left,
                                                                    ret.trace_output,traces),
                Ok(_) => tracer.trace_failed_call(ret.trace_info, traces, vm::Error::Reverted.into()),
                Err(ref e) => tracer.trace_failed_call(ret.trace_info, traces, e.into()),
            };
        
            trace!(target: "executive", "substate={:?}; uncomfirmed_substate={:?} \n", substate, uncon_sub);
            self.inner.enact_result(&res, substate, uncon_sub);
            trace!(target: "executive", "enacted: substate={:?} \n", substate);
            res
        } else {
            self.inner.state.discard_checkpoint();
            tracer.trace_call(ret.trace_info, U256::zero(), ret.trace_output, vec![]);
            Ok(evm::FinalizationResult {
                gas_left: params.gas,
                return_data: ReturnData::empty(),
                apply_state: true
            })
        }
    }

    fn init_vm(ext: &(impl ExternalitiesExt + vm::Ext),
               schedule: Schedule, params: ActionParams, 
               vm_factory: VmFactory
    ) -> err::Result<(Box<VMEmulator + Send + Sync>, rayon::ThreadPool)> {

        let local_stack_size = LOCAL_STACK_SIZE.with(|sz| sz.get());
        let depth_threshold = local_stack_size.saturating_sub(STACK_SIZE_ENTRY_OVERHEAD) / STACK_SIZE_PER_DEPTH;
        
        trace!(target: "executive", "ext.schedule.have_delegate_call: {}", 
        ext.schedule().have_delegate_call);
        let vm: Box<VMEmulator + Send + Sync> = vm_factory.create_debug(params, ext);
        
        let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(2)
                    .stack_size(std::cmp::max(schedule.max_depth.saturating_sub(depth_threshold) * STACK_SIZE_PER_DEPTH, local_stack_size))
                    .build()?;
        Ok((vm, pool))
    }
    
    fn debug_resume(action: Action, ext: &mut (impl ExternalitiesExt + Send), vm: &mut Arc<VMEmulator + Send + Sync>, pool: &rayon::ThreadPool
    ) -> err::Result<ExecInfo> { // should return ExecInfo

        Ok(pool.install(move ||
                Arc::get_mut(vm).unwrap().fire(action, ext)
            )?)
    }


    delegate! {
        target self.inner {
            // pub fn new(state: &'a mut State<B>, info ; &'a EnvInfo, machine: &'a Machine) -> Self;

            pub fn finalize<T,V>(&mut self, 
                                 t: &SignedTransaction, 
                                 substate: Substate, 
                                 result: vm::Result<evm::FinalizationResult>, 
                                 output: Bytes, 
                                 trace: Vec<T>, 
                                 vm_trace: Option<V>
            ) -> Result<Executed<T, V>, ExecutionError>;

            pub fn create<T, V>(
                    &mut self,
                    params: ActionParams,
                    substate: &mut Substate,
                    output: &mut Option<Bytes>,
                    tracer: &mut T,
                    vm_tracer: &mut V
                ) -> vm::Result<evm::FinalizationResult> where T: Tracer, V: VMTracer;
        }
    }
}

impl<'a, B:  'a + StateBackend> ExecutiveExt<'a, B> for Executive<'a, B> {
    delegate! {
        target self.inner {
            fn as_dbg_externalities<'any, T, V>(&'any mut self,
                                                origin_info: OriginInfo,
                                                substate: &'any mut Substate,
                                                output: OutputPolicy<'any, 'any>,
                                                tracer: &'any mut T,
                                                vm_tracer: &'any mut V,
                                                static_call: bool
            ) -> DebugExt<'any, T, V, B> where T: Tracer, V: VMTracer;

            fn _transact_debug(&mut self, 
                                    t: &SignedTransaction,
                                    check_nonce: bool
            ) -> err::Result<(Address, U256, U512)>;

            fn _debug_call<T, V>(&mut self,
                                params: &ActionParams,
                                substate: &mut Substate,
                                tracer: &mut T,
                                vm_tracer: &mut V
            ) -> vm::Result<DebugReturn<T, V>>
                where T: Tracer, V: VMTracer;
                   
             fn _end_call<T, V>(&mut self, 
                          tracer: &mut T,
                          vm_tracer: &mut V,
                          trace_output: Option<Bytes>,
                          trace_info: Option<Call>,
                          subtracer: T,
                          subvmtracer: V,
                          res: vm::Result<evm::FinalizationResult>,
                          substate: &mut Substate,
                          unconfirmed_substate: Substate,
                          gas: U256
            ) -> vm::Result<evm::FinalizationResult>
                where T: Tracer, V: VMTracer;
        }
    }
}
