use vm::{Schedule, GasLeft, ActionParams};
use ethcore::state::{Substate};
use ethcore::trace::trace::Call;
use ethcore::trace::{Tracer, VMTracer};
use ethereum_types::{U256};
use bytes::{Bytes, BytesRef};
use std::sync::Arc;
use crate::emulator::VMEmulator;

crate struct NewBytes(Bytes);

impl NewBytes {
  fn bytes(&self) -> Bytes {
    self.0.clone()
  }
}

impl<'a> From<BytesRef<'a>> for NewBytes {
  fn from(bytes: BytesRef<'_>) -> NewBytes {
    match bytes {
      BytesRef::Flexible(bytes) => NewBytes(Vec::from(bytes.as_mut_slice())),
      BytesRef::Fixed(bytes) => NewBytes(Vec::from(bytes)),
    }
  }
}

// TODO: Condense these into one struct that is a builder #p2
  // https://stackoverflow.com/questions/28951503/how-can-i-create-a-function-with-a-variable-number-of-arguments
  // then I can define a trait with shared funcs

crate struct FinalizeNoCode {
    crate trace_info: Option<Call>,
    crate trace_output: Option<Bytes>,
    crate gas_given: U256
}

impl FinalizeNoCode {
    crate fn new(trace_info: Option<Call>, trace_output: Option<Bytes>, gas_given: U256) -> Self {
        FinalizeNoCode {
            trace_info, trace_output, gas_given
        }
    }
}

crate struct FinalizeInfo<T: Tracer, V: VMTracer>
{
    crate gas: Option<vm::Result<GasLeft>>,
    crate subtracer: T,
    crate subvmtracer: V,
    crate trace_info: Option<Call>,
    crate trace_output: Option<Bytes>,
    crate unconfirmed_substate: Substate,
    crate is_code: bool
}

impl<T, V> FinalizeInfo<T, V> 
    where T: Tracer,
          V: VMTracer,
{
    crate fn new(gas: Option<vm::Result<GasLeft>>, 
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
      OutputPolicy::Return(BytesRef::from(self.output), self.tracej_output.as_mut())
    }
    */
}

crate struct TransactInfo<T: Tracer, V: VMTracer> {
  pub tracer: T,
  pub vm_tracer: V,
  output: Bytes,
  substate: Substate,
  params: ActionParams,
}

impl<T,V> TransactInfo<T,V> where T: Tracer, V: VMTracer {
  crate fn new(tracer: T, vm_tracer: V, output: Bytes, substate: Substate, params: ActionParams) -> Self  {
    TransactInfo {
      tracer, vm_tracer, output, substate, params
    }
  }

  crate fn params(&self) -> &ActionParams {
    &self.params
  }
}

crate struct ResumeInfo {
    vm: Arc<dyn VMEmulator + Send + Sync>,
    pool: rayon::ThreadPool,
}

// need to create Externalities in layer above Executive and pass it in to things that need it
impl ResumeInfo {

    pub fn new(vm: Arc<dyn VMEmulator + Send + Sync>, 
               pool: rayon::ThreadPool
    ) -> Self {
        ResumeInfo {
          vm,pool
        }
    }

    pub fn pool(&mut self) -> &rayon::ThreadPool {
      &self.pool
    }

    pub fn vm(&mut self) -> Arc<dyn VMEmulator + Send + Sync> {
      self.vm.clone()
    }
}
