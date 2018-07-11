use evm::interpreter::Interpreter;
use evm::{CostType};

// 0 state is before interpreter did anything
struct InterpreterSnapshot<Cost: CostType> {
    states: Vec<Interpreter<Cost>>,
}

//impl
//
// mod client_ext;
mod executive_ext;
pub mod interpreter_ext;

