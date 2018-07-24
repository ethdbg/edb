use extensions::ExecutiveExt;
use extensions::executive_utils::FinalizeInfo;
use ethcore::trace::{Tracer, VMTracer};
pub struct DebugExecutive<T: Tracer, V: VMTracer> {
  finalization_info: Option<FinalizeInfo<T, V>>,

}

impl<T, V> DebugExecutive<T,V> where T: Tracer, V: VMTracer {
    fn new() -> Self {
    }
}
