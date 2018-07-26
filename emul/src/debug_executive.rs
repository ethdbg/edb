use vm::{EnvInfo, Schedule};
use ethcore::externalities::*;
use ethcore::executive::{Executive, TransactOptions};
use ethcore::executed::{Executed};
use ethcore::state::{Backend as StateBackend, State};
use ethcore::machine::EthereumMachine as Machine;
use ethcore::trace::{Tracer, VMTracer};
use transaction::SignedTransaction;
use crate::extensions::executive_utils::{FinalizeInfo, ResumeInfo, FinalizeNoCode, TransactInfo};
use crate::extensions::executive_ext::{ExecutiveExt, ExecutionState, CallState};
use crate::extensions::ExecInfo;
use crate::externalities::ExternalitiesExt;
use crate::err::{Error, DebugError};

use std::borrow::BorrowMut;
// acts as a state machine for transaction execution
// continually trying to reach the next Debug State until `finish()` can be called

enum FinalizeType {
    NoCode(FinalizeNoCode), // a transaction that does not require code to be executed (basic tx)
    Code(FinalizeInfo),    // tx that requires execution
    Create(vm::Result<evm::FinalizationResult>) // create a contract -- no debugging capability yet
}

enum DebugState {
    Resumable(ResumeInfo, TransactInfo, FinalizeType), // -- execution can be manipulated
    NeedsFinalization(FinalizeType, TransactInfo), // a transaction that needs to be finalized
    Done(vm::Result<evm::FinalizationResult>), // a transaction that is finished
    Nil, // if unwrapped to `None` value, this is a default
}

trait DebugFields {
    fn tx_info(&self) -> crate::err::Result<&TransactInfo>;
    fn fin_info(&mut self) -> crate::err::Result<&mut FinalizeInfo>;
    fn is_resumable(&self) -> bool;
}
const dbg_err_str: &'static str = "DebugState or DebugExecution Object not intitalized; \
                                   but attempt to call a function defined on \
                                   `Debug Fields` occured anyway";

// defaults and error handling for Option<> fields on DebugExecutive
impl DebugFields for Option<DebugExecution> {
    fn tx_info(&self) -> crate::err::Result<&TransactInfo> {
        let err_str = "Attempt to get Transaction Info from a state \
                       where Transaction Info does not exist";

        match self.map(|s| s.state).ok_or(Error::Debug(DebugError::from(dbg_err_str)))? {
            DebugState::Resumable(_,txinfo,_) => Ok(&txinfo),
            DebugState::NeedsFinalization(_, txinfo) => Ok(&txinfo),
            _=> Err(Error::Debug(DebugError::from(err_str)))
        }
    }

    fn fin_info(&mut self) -> crate::err::Result<&mut FinalizeInfo> {

        let error_str = "Attempt to get Finalization Information from an object that was not `Resumable`";

        match self.map(|s| s.state).ok_or(Error::Debug(DebugError::from(dbg_err_str)))? {
            DebugState::Resumable(_, _, fin_type) => {
                match fin_type {
                    FinalizeType::Code(fin_info) => Ok(&mut fin_info),
                    _=> Err(Error::Debug(DebugError::from(error_str)))
                }
            },
            _ => Err(Error::Debug(DebugError::from(error_str)))
        }
    }

    fn is_resumable(&self) -> bool {
        match self.map(|s| s.state).unwrap_or(DebugState::Nil) {
            DebugState::Resumable(_,_,_) => true,
            _ => false
        }
    }
}

pub struct DebugExecution {
    state: DebugState,
}

impl DebugExecution {

    fn new<'a, B, T, V>(t: &SignedTransaction, 
           options: TransactOptions<T,V>, 
           executive: impl ExecutiveExt<'a, B>,
    ) -> crate::err::Result<Self> 
        where B: 'a + StateBackend,
              T: Tracer,
              V: VMTracer
    {

        let state = match executive.transact_debug(t, options)? {
            ExecutionState::Create(res, txinfo) => DebugState::NeedsFinalization(FinalizeType::Create(res), txinfo),
            ExecutionState::Call(call_state, txinfo) => {
                match call_state {
                    CallState::Called(fin_info, resume_info) => {
                        DebugState::Resumable(resume_info, txinfo, FinalizeType::Code(fin_info))
                    },
                    CallState::NoCodeCall(fin_info) => 
                        DebugState::NeedsFinalization(FinalizeType::NoCode(fin_info), txinfo)
                }

            }
        };

        Ok(DebugExecution {
            state,
        })
    }
}
pub struct DebugExecutive<'a, B: 'a + StateBackend, E: ExternalitiesExt + vm::Ext> {
    inner: Executive<'a, B>,
    tx: Option<DebugExecution>,
    ext: Option<Box<E>>
}

impl<'a,B,E> DebugExecutive<'a,B,E> 
where B: 'a + StateBackend,
      E: ExternalitiesExt + vm::Ext
{
    pub fn new(state: &'a mut State<B>, 
        info: &'a EnvInfo,
        machine: &'a Machine, 
        schedule: &'a Schedule
    ) -> Self {

        DebugExecutive {
            inner: Executive::new(state, info, machine, schedule),
            tx: None,
            ext: None,
        }
    }
 
    pub fn begin_transact<T,V>(&'a mut self, t: &SignedTransaction, options: TransactOptions<T,V>
    ) -> crate::err::Result<()> where T: Tracer, V: VMTracer
    {
        self.tx = Some(DebugExecution::new(t, options, self.inner)?);

        if self.tx.is_resumable() {
            let txinfo = self.tx.tx_info()?;
            let fininfo = self.tx.fin_info()?;
            let ext =
                self.inner.as_dbg_externalities(
                    OriginInfo::from(txinfo.params()),
                    &mut fininfo.unconfirmed_substate,
                    OutputPolicy::Return,
                    txinfo.tracer.borrow_mut(),
                    txinfo.vm_tracer.borrow_mut(),
                    txinfo.params().call_type == evm::CallType::StaticCall

            );
        }

        Ok(())
    }

    pub fn resume(&mut self) -> crate::err::Result<ExecInfo> {
        unimplemented!();
    }

    pub fn finish<T,V>(&mut self) -> crate::err::Result<Executed<T::Output, V::Output>> where T:Tracer, V:VMTracer {
        self.tx = None;
        self.ext = None;
        unimplemented!();
    }
}
