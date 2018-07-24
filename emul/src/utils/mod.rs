use {vm, rayon};
use vm::{Schedule, GasLeft};
use ethcore::state::{Substate, Backend as StateBackend};
use ethcore::trace::trace::Call;
use ethcore::trace::{Tracer, VMTracer};
use ethereum_types::{U256};
use bytes::Bytes;
use std::sync::Arc;
use externalities::{ConsumeExt, ExternalitiesExt};
use emulator::VMEmulator;


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


pub struct FinalizeNoCode<T: Tracer> 
{
    pub tracer: T,
    pub trace_info: Option<Call>,
    pub trace_output: Option<Bytes>,
    pub gas_given: U256
}

impl<T> FinalizeNoCode<T> where T: Tracer {
    pub fn new(tracer: T, trace_info: Option<Call>, trace_output: Option<Bytes>, gas_given: U256) -> Self {
        FinalizeNoCode {
            tracer, trace_info, trace_output, gas_given
        }
    }
}

pub struct FinalizeInfo<T: Tracer, V: VMTracer>
{
    pub gas: Option<vm::Result<GasLeft>>,
    pub tracer: T,
    pub vm_tracer: V,
    pub subtracer: T,
    pub subvmtracer: V,
    pub trace_info: Option<Call>,
    pub trace_output: Option<Bytes>,
    pub gas_given: U256,
    pub substate: Substate,
    pub unconfirmed_substate: Substate,
    pub is_code: bool
}

impl<T, V> FinalizeInfo<T, V> 
    where T: Tracer,
          V: VMTracer,
{
    pub fn new(gas: Option<vm::Result<GasLeft>>, 
               tracer: T, 
               vm_tracer: V, 
               subvmtracer: V, 
               subtracer: T,
               trace_info: Option<Call>, 
               trace_output: Option<Bytes>, 
               gas_given: U256, 
               substate: Substate, 
               unconfirmed_substate: Substate, 
               is_code: bool) -> Self {

        FinalizeInfo {
            gas, tracer, vm_tracer, subvmtracer, subtracer, trace_info, trace_output, gas_given, substate, unconfirmed_substate, is_code
        }
    }
}

pub struct ResumeInfo<E: ExternalitiesExt + vm::Ext> {
    ext: E,
    vm: Arc<VMEmulator + Send + Sync>,
    pool: rayon::ThreadPool,
}

impl<E> ResumeInfo<E> where E: ExternalitiesExt + vm::Ext {

    pub fn new(ext: E, 
               vm: Arc<VMEmulator + Send + Sync>, 
               pool: rayon::ThreadPool
    ) -> Self {
        ResumeInfo {
            ext,vm,pool
        }
    }

}
