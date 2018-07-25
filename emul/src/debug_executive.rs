use ethcore::executive::{Executive, TransactOptions};
use ethcore::executed::{Executed};
use ethcore::state::{Backend as StateBackend, State};
use ethcore::machine::EthereumMachine as Machine;
use vm::{EnvInfo, Schedule};
use transaction::SignedTransaction;
use crate::extensions::executive_utils::FinalizeInfo;
use crate::extensions::executive_ext::ExecutiveExt;
use crate::extensions::ExecInfo;
use ethcore::trace::{Tracer, VMTracer};

pub struct DebugExecutive<'a, T: Tracer, V: VMTracer, B: 'a + StateBackend> {
    inner: Executive<'a, B>,
    finalization_info: Option<FinalizeInfo<T, V>>,
}

impl<'a, T, V, B> DebugExecutive<'a, T,V,B> where T: Tracer, V: VMTracer, B: 'a + StateBackend {

    pub fn new(state: &'a mut State<B>, 
        info: &'a EnvInfo,
        machine: &'a Machine, 
        schedule: &'a Schedule
    ) -> Self {

        DebugExecutive {
            finalization_info: None,
            inner: Executive::new(state, info, machine, schedule)
        }
    }
    pub fn begin_transact(&mut self, t: &SignedTransaction, options: TransactOptions<T,V>) {


    }

    pub fn resume(&mut self) -> crate::err::Result<ExecInfo> {
        unimplemented!();
    }

    pub fn finish(&mut self
    ) -> crate::err::Result<Executed<T::Output, V::Output>> where T: Tracer, V: VMTracer {
        unimplemented!();
    }
}
