#![feature(use_extern_macros)]
#![feature(proc_macro_gen)]
#[macro_use] mod tests;
pub mod emulator;
mod err;
mod scary;

pub use self::emulator::Action;
pub use sputnikvm::ValidTransaction;
pub use sputnikvm::HeaderParams;
