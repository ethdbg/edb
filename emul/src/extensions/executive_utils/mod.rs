use {vm, rayon};
use vm::{Schedule, GasLeft, ActionParams};
use ethcore::state::{Substate};
use ethcore::trace::trace::Call;
use ethcore::trace::{Tracer, VMTracer};
use ethereum_types::{U256};
use bytes::{Bytes, BytesRef};
use std::sync::Arc;
use emulator::VMEmulator;



pub struct NewBytes(Bytes);

impl NewBytes {
  fn bytes(&self) -> Bytes {
    self.0.clone()
  }
}

impl<'a> From<BytesRef<'a>> for NewBytes {
  fn from(bytes: BytesRef) -> NewBytes {
    match bytes {
      BytesRef::Flexible(bytes) => NewBytes(Vec::from(bytes.as_mut_slice())),
      BytesRef::Fixed(bytes) => NewBytes(Vec::from(bytes)),
      _=> panic!("Unknown Bytes Type Conversion!")
    }
  }
}


pub struct DebugReturn<T: Tracer, V: VMTracer> {
    pub schedule: Option<Schedule>,
    pub unconfirmed_substate: Option<Substate>,
    pub trace_output: Option<Bytes>,
    pub trace_info: Option<Call>,
    pub subtracer: Option<T>,
    pub subvmtracer: Option<V>,
    pub is_code: bool
}

impl<T: Tracer, V: VMTracer> DebugReturn<T, V> {
    pub fn is_code(&self) -> bool {
        self.is_code
    }
}


pub struct FinalizeNoCode {
    pub trace_info: Option<Call>,
    pub trace_output: Option<Bytes>,
    pub gas_given: U256
}

impl FinalizeNoCode {
    pub fn new(trace_info: Option<Call>, trace_output: Option<Bytes>, gas_given: U256) -> Self {
        FinalizeNoCode {
            trace_info, trace_output, gas_given
        }
    }
}

pub struct FinalizeInfo<T: Tracer, V: VMTracer>
{
    pub gas: Option<vm::Result<GasLeft>>,
    pub subtracer: T,
    pub subvmtracer: V,
    pub trace_info: Option<Call>,
    pub trace_output: Option<Bytes>,
    pub unconfirmed_substate: Substate,
    pub is_code: bool
}

impl<T, V> FinalizeInfo<T, V> 
    where T: Tracer,
          V: VMTracer,
{
    pub fn new(gas: Option<vm::Result<GasLeft>>, 
               subvmtracer: V, 
               subtracer: T,
               trace_info: Option<Call>, 
               trace_output: Option<Bytes>, 
               unconfirmed_substate: Substate, 
               is_code: bool) -> Self {

        FinalizeInfo {
            gas, subvmtracer, 
            subtracer, trace_info, trace_output, unconfirmed_substate, is_code
        }
    }
    /*
    pub fn is_static(&self) -> bool {
      self.params.call_type == CallType::StaticCall
    }
    */
/*
    pub fn boxed_output_policy<'any>(&self) -> OutputPolicy<'any, 'any> {
      OutputPolicy::Return(BytesRef::from(self.output), self.trace_output.as_mut())
    }
    */
}

pub struct TransactInfo<T: Tracer, V: VMTracer> {
  tracer: T,
  vm_tracer: V,
  output: Bytes,
  substate: Substate,
  params: ActionParams,
}

impl<T,V> TransactInfo<T,V> where T: Tracer, V: VMTracer {
  pub fn new(tracer: T, vm_tracer: V, output: Bytes, substate: Substate, params: ActionParams) -> Self  {
    TransactInfo {
      tracer, vm_tracer, output, substate, params
    }
  }
}

pub struct ResumeInfo {
    vm: Arc<VMEmulator + Send + Sync>,
    pool: rayon::ThreadPool,
}

// need to create Externalities in layer above Executive and pass it in to things that need it
impl ResumeInfo {

    pub fn new(vm: Arc<VMEmulator + Send + Sync>, 
               pool: rayon::ThreadPool
    ) -> Self {
        ResumeInfo {
          vm,pool
        }
    }

}
