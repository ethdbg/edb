use tokio::prelude::*;
use ethcore::client::{Client, CallAnalytics};
use ethcore::executed::Executed;
use ethcore::executive::TransactOptions;
use transaction::SignedTransaction;
use ethcore::machine::EthereumMachine as Machine;
use ethcore::state_db::StateDB;
use ethcore::state::State;
use ethcore::error::CallError;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;

use vm::EnvInfo;
use extensions::ExecInfo;
use executive::Executive;
use emulator::Action;
use err::Error;
use debug_handler::DebugHandler;

/*
pub enum ExecutionState {
    Executing(ExecInfoFuture),
    Done(ExecutedFuture)
}

struct ExecInfoFuture;
struct ExecutedFuture;
struct ExecuteAndDoneFuture;

impl Future for ExecInfoFuture {
    type Item = ExecInfo;
    type Error;

    fn poll(&mut self) -> Result<Async<ExecInfo>, ()> {
        // try_recv() etc
    }
}

impl Future for ExecutedFuture {
    type Item = Executed;
    type Error = CallError;

    fn poll(&mut self) -> Result<Async<ExecutedFuture>, CallError> {

    }
}

impl Future for ExecuteAndDoneFuture {

    fn poll(&mut self) -> Result<Async<()>, ()> {
        
    }
}
*/
pub trait DebugClient<'a> {
    fn start_debug_call(machine: Arc<Machine>, 
                        env_info: Arc<EnvInfo>, 
                        state: &'static mut State<StateDB>, 
                        t: Arc<SignedTransaction>, 
                        analytics: CallAnalytics,
                        debug_handler: &mut DebugHandler<Executed>) -> Result<(), CallError>;
    fn resume_debug_call(debug_handler: &DebugHandler<Executed>, action: Action) -> Result<ExecInfo, Error>;
    fn finish_debug_call(debug_handler: DebugHandler<Executed>) -> Result<Executed, CallError>;
}

/* 
    - two channels here
    - one for sending back ExecInfo
    - one for sending Resume calls
    - `multiple producer, multiple consumer` type thing
    - I chose this method because UNIX sockets won't work on Windows,
        - and named pipes for both would be a bit janky (some unified interface would need to be
        created)


    MPSC channels created somewhere.....
        where? xD
    start_debug_call will spawn thread, so channels need to be there
*/

/// these three functions must be called in order; or else a panic will occur.
/// Initialize creates spawns thread an initializes channels for communication
/// resume can be used as many times as the transaction needs to progress
/// ends will 
impl<'a> DebugClient<'a> for Client {

    fn start_debug_call(machine: Arc<Machine>, 
                        env_info: Arc<EnvInfo>, 
                        state: &'static mut State<StateDB>, 
                        t: Arc<SignedTransaction>, 
                        analytics: CallAnalytics,
                        debug_handler: &mut DebugHandler<Executed>
    ) -> Result<(), CallError> {

        // can do all other forms of tracing, too. for now this is enough.
        let original_state = if analytics.state_diffing { Some(state.clone()) } else { None };
        let options = Arc::new(TransactOptions::with_tracing_and_vm_tracing()
            .dont_check_nonce()
            .save_output_from_contract());
        let (atx, arx) : (Sender<Action>, Receiver<Action>) = mpsc::channel();
        let (etx, erx) : (Sender<ExecInfo>, Receiver<ExecInfo>) = mpsc::channel();
        

        debug_handler.init(atx, erx, 
                           move || { 
                               let res = Executive::new(state, env_info.clone(), machine.clone()).transact_debug(t.clone(), options, etx, arx).unwrap();
                               res
                           });
        Ok(())
    }
    
    fn resume_debug_call(debug_handler: &DebugHandler<Executed>, action: Action) -> Result<ExecInfo, Error> {
        debug_handler.send(action);
        Ok(debug_handler.recv()?)
    }

    fn finish_debug_call(debug_handler: DebugHandler<Executed>) -> Result<Executed, CallError> {
        Ok(debug_handler.join())
    }
}
