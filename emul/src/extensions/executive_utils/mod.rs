use vm::{Schedule, GasLeft, ActionParams};
use ethcore::state::{Substate};
use ethcore::trace::trace::Call;
use ethcore::trace::{Tracer, VMTracer, FlatTrace, VMTrace};
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


crate struct DebugReturn {
    crate schedule: Option<Schedule>,
    crate unconfirmed_substate: Option<Substate>,
    crate trace_output: Option<Bytes>,
    crate trace_info: Option<Call>,
    crate subtracer: Option<Box<dyn Tracer<Output=FlatTrace>>>,
    crate subvmtracer: Option<Box<dyn VMTracer<Output=VMTrace>>>,
    crate is_code: bool
}

impl DebugReturn {
    crate fn is_code(&self) -> bool {
        self.is_code
    }
}


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

crate struct FinalizeInfo {
    crate gas: Option<vm::Result<GasLeft>>,
    crate subtracer: Box<dyn Tracer<Output=FlatTrace>>,
    crate subvmtracer: Box<dyn VMTracer<Output=VMTrace>>,
    crate trace_info: Option<Call>,
    crate trace_output: Option<Bytes>,
    crate unconfirmed_substate: Substate,
    crate is_code: bool
}

impl FinalizeInfo {
    crate fn new(gas: Option<vm::Result<GasLeft>>, 
               subvmtracer: impl VMTracer,
               subtracer: impl Tracer,
               trace_info: Option<Call>, 
               trace_output: Option<Bytes>, 
               unconfirmed_substate: Substate, 
               is_code: bool) -> Self {

        FinalizeInfo {
            gas, 
            subvmtracer: Box::new(subvmtracer),
            subtracer: Box::new(subtracer), 
            trace_info, 
            trace_output, 
            unconfirmed_substate, 
            is_code
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

crate struct TransactInfo {
  pub tracer: Box<dyn Tracer<Output=FlatTrace>>,
  pub vm_tracer: Box<dyn VMTracer<Output=VMTrace>>,
  output: Bytes,
  substate: Substate,
  params: ActionParams,
}

impl TransactInfo {
  crate fn new(tracer: Box<dyn Tracer<Output=FlatTrace>>, vm_tracer: Box<dyn VMTracer<Output=VMTrace>>, output: Bytes, substate: Substate, params: ActionParams) -> Self  
  {
    TransactInfo {
      tracer: tracer,
      vm_tracer: vm_tracer,
      output, substate, params
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

}
