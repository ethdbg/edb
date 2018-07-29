use vm::{EnvInfo, Schedule};
use ethcore::externalities::*;
use ethcore::executive::{Executive, TransactOptions};
use ethcore::executed::{Executed};
use ethcore::state::{Backend as StateBackend, State};
use ethcore::machine::EthereumMachine as Machine;
use ethcore::trace::{Tracer, VMTracer};
use transaction::SignedTransaction;
use std::mem;
use crate::extensions::executive_utils::{FinalizeInfo, ResumeInfo, FinalizeNoCode, TransactInfo, debug_resume};
use crate::extensions::executive_ext::{ExecutiveExt, ExecutionState, CallState};
use crate::extensions::ExecInfo;
use crate::externalities::ExternalitiesExt;
use crate::err::{Error, DebugError};
use crate::emulator::Action;
use crate::err;
// acts as a state machine for transaction execution
// continually trying to reach the next Debug State until `finish()` can be called

enum FinalizeType<T: Tracer, V: VMTracer> {
    NoCode(FinalizeNoCode), // a transaction that does not require code to be executed (basic tx)
    Code(FinalizeInfo<T,V>),    // tx that requires execution
    Create(vm::Result<evm::FinalizationResult>) // create a contract -- no debugging capability yet
}

enum DebugState <T: Tracer, V: VMTracer> {
    Resumable(ResumeInfo, TransactInfo<T,V>, FinalizeType<T,V>), // -- execution can be manipulated
    NeedsFinalization(FinalizeType<T,V>, TransactInfo<T,V>), // a transaction that needs to be finalized
    Done(vm::Result<evm::FinalizationResult>), // a transaction that is finished
}

trait Info<T,V> where T: Tracer, V: VMTracer {
    fn fin_info(&mut self) -> crate::err::Result<&mut FinalizeInfo<T,V>>;
    fn tx_info(&mut self) -> crate::err::Result<&mut TransactInfo<T,V>>;
    fn info(&mut self) -> crate::err::Result<(&mut TransactInfo<T,V>, &mut FinalizeInfo<T,V>)>;
    fn resumables(&mut self) -> crate::err::Result<(&mut ResumeInfo, &mut TransactInfo<T,V>, &mut FinalizeType<T,V>)>;
}

//  Match statements and error handling
impl<T,V> Info<T,V> for DebugState<T,V> where T: Tracer, V: VMTracer {
    fn tx_info(&mut self) -> crate::err::Result<&mut TransactInfo<T,V>> {
        let err_str = "Attempt to get Transaction Info from a state \
                       where Transaction Info does not exist";
        match self {
            DebugState::Resumable(_,txinfo,_) => Ok(txinfo),
            DebugState::NeedsFinalization(_,txinfo) => Ok(txinfo),
            _=> Err(Error::Debug(DebugError::from(err_str)))
        }
    }

    fn fin_info(&mut self) -> crate::err::Result<&mut FinalizeInfo<T,V>> {
        let err_str = "Attempt to get Finalization Info from a state \
                        where Finalization Info does not exist, or is not `Resumable`";
        match self {
            DebugState::Resumable(_, _, fin_type) => {
                match fin_type {
                    FinalizeType::Code(fin_info) => Ok(fin_info),
                    _=> Err(Error::Debug(DebugError::from(err_str)))
                }
            },
            _ => Err(Error::Debug(DebugError::from(err_str)))
        }
    }

    fn info(&mut self) -> crate::err::Result<(&mut TransactInfo<T,V>, &mut FinalizeInfo<T,V>)> {
        let err_str = "Attempt to get Finalize Info and Transact Info, but either state is not `Resumable` or not initialized";

        match self {
            DebugState::Resumable(_, txinfo, fin_type) => {
                match fin_type {
                    FinalizeType::Code(fin_info) => Ok((txinfo, fin_info)),
                    _=> Err(Error::Debug(DebugError::from(err_str)))
                }
            }
            _=> Err(Error::Debug(DebugError::from(err_str)))
        }
    }

    fn resumables(&mut self) -> crate::err::Result<(&mut ResumeInfo, &mut TransactInfo<T,V>, &mut FinalizeType<T,V>)> {
        let err_str = "State must be `Resumable` in order to obtain Resumeables";
        match self {
            DebugState::Resumable(resume_info, tx_info, fin_type) => Ok((resume_info, tx_info, fin_type)),
            _=> Err(Error::Debug(DebugError::from(err_str)))
        }
    }
}

trait DebugFields<T: Tracer, V: VMTracer>: Sized {
    fn tx_info<F>(&mut self, f: F) -> err::Result<()> where F: FnMut(&mut TransactInfo<T,V>) -> err::Result<()>;
    fn fin_info<F>(&mut self, f: F) -> err::Result<()> where F: FnMut(&mut FinalizeInfo<T,V>) -> err::Result<()>;
    fn info<F>(&mut self, f: F) -> err::Result<()>
        where F: FnMut(&mut TransactInfo<T,V>, &mut FinalizeInfo<T,V>) -> err::Result<()>;
    fn resumables<F>(&mut self, f: F) -> err::Result<()>
        where F: FnMut(&mut ResumeInfo, &mut TransactInfo<T,V>, &mut FinalizeType<T,V>) -> crate::err::Result<()>;
    fn with_ext<'a, B, F>(&mut self, f: F, executive: &mut impl ExecutiveExt<'a, B>) -> err::Result<()> 
        where F: FnMut(&mut dyn ExternalitiesExt) -> err::Result<()>,
        B: 'a + StateBackend;
    fn with_resumables<'a, B, F>(&mut self, f: F, executive: &mut impl ExecutiveExt<'a, B>) -> err::Result<()>
        where F: FnMut(&mut (dyn ExternalitiesExt + Send), &mut ResumeInfo) -> err::Result<()>, B: 'a + StateBackend;
    fn update(&mut self, state: DebugState<T,V>);
    fn can_finish(&self) -> bool;
    fn is_resumable(&self) -> bool;
}

/// defaults and error handling for Option<> fields on DebugExecutive
/// higher order functions for using data in State
/// consumes the Option<>
impl<T,V> DebugFields<T,V> for Option<DebugExecution<T,V>> 
    where T: Tracer, V: VMTracer
{   
    /// use TransactInfo by-mutable-reference
    fn tx_info<F>(&mut self, mut f: F) -> err::Result<()>
    where 
        F: FnMut(&mut TransactInfo<T,V>) -> err::Result<()> 
    {
        
        let err_str = "Attempt to get Transaction Info from struct `DebugExecution` that was not yet initialized";
        self.as_mut().take().map(|s| s.state())
                .ok_or(Error::Debug(DebugError::from(err_str)))
                .and_then(Info::tx_info).iter_mut().map(|mut t| f(&mut t)).collect()
    }

    /// use finalization info by-mutable-reference
    fn fin_info<F>(&mut self, mut f: F) -> err::Result<()>
    where 
        F: FnMut(&mut FinalizeInfo<T,V>) -> crate::err::Result<()> 
    {
        let err_str = "Attempt to get Finalization Info from struct `DebugExecution` that was not yet initialized";
        self.as_mut().take().map(|s| s.state())
            .ok_or(Error::Debug(DebugError::from(err_str)))
            .and_then(Info::fin_info).iter_mut().map(|mut fin| f(&mut fin)).collect()
    }

    /// use both finalization info and transact info by mutable reference
    fn info<F>(&mut self, mut f: F) -> err::Result<()>
    where 
        F: FnMut(&mut TransactInfo<T,V>, &mut FinalizeInfo<T,V>) -> crate::err::Result<()>
    {       
        let err_str = "Attempt to get Finalization Info from struct `DebugExecution` \
                      that was not yet initialized";
        self.as_mut().take().map(|s| s.state())
            .ok_or(Error::Debug(DebugError::from(err_str)))
            .and_then(Info::info).iter_mut().map(|i| f(i.0, i.1)).collect()
    }
    
    fn resumables<F>(&mut self, mut f: F) -> err::Result<()>
        where F: FnMut(&mut ResumeInfo, &mut TransactInfo<T,V>, &mut FinalizeType<T,V>) -> crate::err::Result<()>
    {   
        let err_str = "fill this in";
        self.as_mut().take().map(|s| s.state())
            .ok_or(Error::Debug(DebugError::from(err_str)))
            .and_then(Info::resumables).iter_mut().map(|i| f(i.0, i.1, i.2)).collect()
    }

    fn with_ext<'a, B, F>(&mut self, mut f: F, executive: &mut impl ExecutiveExt<'a, B>) -> err::Result<()> 
        where F: FnMut(&mut dyn ExternalitiesExt) -> crate::err::Result<()>, B: 'a + StateBackend
    {
        self.info(|txinfo, fin_info| {
            let static_call = txinfo.params().call_type == evm::CallType::StaticCall;
            let mut ext = executive.as_dbg_externalities(OriginInfo::from(txinfo.params()),
                &mut fin_info.unconfirmed_substate,
                OutputPolicy::Return,
                &mut txinfo.tracer,
                &mut txinfo.vm_tracer,
                static_call);
            f(&mut ext)
        })
    }

    fn with_resumables<'a, B, F>(&mut self, mut f: F, executive: &mut impl ExecutiveExt<'a, B>) -> err::Result<()>
        where F: FnMut(&mut (dyn ExternalitiesExt + Send), &mut ResumeInfo) -> err::Result<()>, B: 'a + StateBackend 
    {
        self.resumables(|resume_info, txinfo, fin_type| {
            let fin_info = match fin_type {
                FinalizeType::Code(fin_info) => Ok(fin_info),
                _ => Err(Error::Debug(DebugError::from("Tried to get resumables from a type that did not have FinalizeInfo")))
            }?;

            let static_call = txinfo.params().call_type == evm::CallType::StaticCall;
            let mut ext = executive.as_dbg_externalities(OriginInfo::from(txinfo.params()),
                &mut fin_info.unconfirmed_substate,
                OutputPolicy::Return,
                &mut txinfo.tracer,
                &mut txinfo.vm_tracer,
                static_call);
            f(&mut ext, resume_info)
        })
    }

    fn update(&mut self, state: DebugState<T,V>) {
        mem::replace(self, Some(state.into()));
    } 

    fn is_resumable(&self) -> bool {
        match *self {
            Some(ref v) => {
                match v.state {
                    DebugState::Resumable(_,_,_) => true,
                    _ => false,
                }
            }
            None => false,
        }
    }

    fn can_finish(&self) -> bool {
        match *self {
            Some(ref v) => {
                match v.state {
                    DebugState::Resumable(_,_,_) => false,
                    _ => true,
                }
            },
            None => false
        }
    }
}

pub struct DebugExecution<T: Tracer, V: VMTracer> {
    state: DebugState<T,V>,
}

impl<T,V> DebugExecution<T,V> 
where T: Tracer, V: VMTracer {
    fn new<'a, B: 'a + StateBackend>(t: &SignedTransaction, 
           options: TransactOptions<T,V>, 
           mut executive: impl ExecutiveExt<'a, B>,
    ) -> crate::err::Result<Self> {

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

    fn state(&mut self) -> &mut DebugState<T,V> {
        &mut self.state
    }
}

impl<T,V> From<DebugState<T,V>> for DebugExecution<T,V> where T: Tracer, V: VMTracer {
    fn from(state: DebugState<T,V>) -> DebugExecution<T,V> {
        DebugExecution {
            state,
        }
    }
}

pub struct DebugExecutive<'a, T: Tracer, V: VMTracer, B: 'a + StateBackend> {
    inner: Executive<'a, B>,
    tx: Option<DebugExecution<T,V>>,
}

impl<'a,T: 'a,V: 'a,B> DebugExecutive<'a,T,V,B> 
where T: Tracer, 
      V: VMTracer, 
      B: 'a + StateBackend
{
    pub fn new(state: &'a mut State<B>, 
        info: &'a EnvInfo,
        machine: &'a Machine, 
        schedule: &'a Schedule
    ) -> Self {

        DebugExecutive {
            inner: Executive::new(state, info, machine, schedule),
            tx: None,
        }
    }
 
    pub fn begin_transact(mut self, t: &SignedTransaction, options: TransactOptions<T,V>
    ) -> crate::err::Result<()> {
        self.tx = Some(DebugExecution::new(t, options, self.inner)?);

        if !self.tx.is_resumable() {

        }
        Ok(())
    }


    /// attempts to progress a `resumable` state to `NeedsFinalization`. Should not be called after exec info `is_finished()`
    pub fn resume(&mut self, action: Action) -> crate::err::Result<ExecInfo> {
        let mut exec_info: Option<ExecInfo> = None;
        if self.tx.is_resumable() {
            self.tx.with_resumables(|ext, resume_info| {
                let res = debug_resume(&action, ext, &mut resume_info.vm(), resume_info.pool())?;
                exec_info = Some(res);
                Ok(())
            }, &mut self.inner)?;

            match exec_info {
                Some(ref e) => { 
                    if e.finished() {
                        match self.tx.take().expect("Scope is conditional, `tx.resumable()`; qed").state {
                            DebugState::Resumable(_, tx_info, fin_type) => {
                                self.tx.update(DebugState::NeedsFinalization(fin_type, tx_info));
                            },
                            _ => panic!("Scope is conditional `tx.resumable()` qed")
                        }
                    }
                },
                None => {} // can't progress state if exec info does not exist! TODO: this should never happen, but add an error #p2
            }
        }
        exec_info.ok_or(Error::Debug(DebugError::from("Resume called on a state \
                                                       that was not resumable")))
    }

    pub fn finish(&mut self
    ) -> crate::err::Result<Executed<T::Output, V::Output>> where T: Tracer, V: VMTracer {
        unimplemented!();
    }
}
