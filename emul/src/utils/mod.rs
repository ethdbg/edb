use vm::Schedule;
use ethcore::state::Substate;
use ethcore::trace::trace::Call;
use ethcore::trace::{Tracer, VMTracer};
use bytes::Bytes;

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
