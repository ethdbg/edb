//! All 'Providers' (Shell, RPC, Etc) implemented these functions

use failure::Error;


pub trait Provider {
    fn run() -> Result<(), Error>;
}


impl<T, L> Provider for Shell where T: Transport, L: Language {
    pub fn run(debugger: Debugger<T, L>) -> {
    //xxxx 
    }
}
