//! All 'Providers' (Shell, RPC, Etc) implemented these functions

use failure::Error;


pub trait Provider {
    fn run() -> Result<(), Error>;
}
