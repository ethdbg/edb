use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use emulator::Action;
use extensions::{ExecInfo};
use std::sync::mpsc::{SendError, RecvError};
use err::Error;
use err;

/*
 * Struct handling threads and calls for DebugClient
 *
 */

pub struct DebugHandler<T: Send + 'static> {
    thread: Option<thread::JoinHandle<T>>,
    sender: Option<mpsc::Sender<Action>>,
    receiver: Option<mpsc::Receiver<ExecInfo>>,
}

impl<T> DebugHandler<T> 
    where T: Send + 'static
{
    pub fn new() -> Self {
        //let (atx, arx) = mpsc::channel();
        //let (etx, erx) = mpsc::channel();

        /** 
        * atx - action sender
        * arx - action receiver
        * etx - execution sender
        * erx - execution receiver
        * client_mpsc is passed to client thread
        * exec_mpsc is passed to the thread we execute (child)
        * */
        DebugHandler {
            thread: None,
            sender: None,
            receiver: None,
        }
    }
    
    pub fn init<F>(&mut self, tx: Sender<Action>, rx: Receiver<ExecInfo>, func: F) 
        where F: FnOnce() -> T, F: Send + 'static
    {
        self.thread = Some(thread::spawn(func));
    }
    
    pub fn send(&self, action: Action) -> Result<(), SendError<Action>> {
        match self.sender {
            Some(ref x) => {
                Ok(x.send(action)?)
            },
            None => panic!("Debug Handler not initialized!")
        }
    }

    pub fn recv(&self) -> Result<ExecInfo, Error> {

        match self.receiver.as_ref().expect("Debug Handler not Initialized").recv() {
            Ok(x) => Ok(x),
            Err(err) => Err(Error::from(err))
        }
    }

    pub fn sender(&self) -> &Sender<Action> {
        self.sender.as_ref().expect("Debug Handler not Initialized!")
    }

    pub fn receiver(&self) -> &Receiver<ExecInfo> {
        self.receiver.as_ref().expect("Debug Handler not Initialized!")
    }

    pub fn join(self) -> T {
        drop(self.sender);
        drop(self.receiver);
        self.thread.expect("Thread has not been initialized").join().unwrap()
    }

}
