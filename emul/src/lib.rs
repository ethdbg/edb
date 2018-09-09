#![feature(proc_macro_gen)]
#[cfg(test)] #[macro_use] mod tests;
pub mod emulator;
mod err;
mod scary;

pub use self::emulator::Action;
pub use sputnikvm::ValidTransaction;
pub use sputnikvm::HeaderParams;
pub use web3::{Web3, Transport};
