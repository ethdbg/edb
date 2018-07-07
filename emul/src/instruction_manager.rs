//! Manages Instructions/Traces 
//! shared between Externalities and Interpreter
use ethereum_types::{U256, H256, Address};
use evm::{Instruction};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct InstructionState {
    instruction: Instruction,
    pub pc: usize,
    gas_cost: U256,
    stack_push: Vec<U256>,
    mem_diff: Option<(usize, Vec<u8>)>,
    store_diff: Option<(U256, U256)>,
}

// struct for current info that is being gathered for tracing
// in our case, being gathered for InstructionState Vector
#[derive(Debug, Default)]
pub struct CurrentExecInfo {
    last_inst: u8,
    pc: usize,
    gas_cost: U256,
}

impl CurrentExecInfo {
    pub fn new() -> CurrentExecInfo {
        CurrentExecInfo::default()
    }

    pub fn clear(&mut self) {
        self.last_inst = 0;
        self.pc = 0;
        self.gas_cost = U256::zero();
    }

}

#[derive(Default, Debug)]
pub struct InstructionManager {
    pub inst_hist: Rc<RefCell<Vec<InstructionState>>>,
    pub curr_exec: Rc<RefCell<CurrentExecInfo>>,
}

impl InstructionManager {

    pub fn new() -> InstructionManager {
        InstructionManager {
            inst_hist: Rc::new(RefCell::new(Vec::new())),
            curr_exec: Rc::new(RefCell::new(CurrentExecInfo::default())),
        }
    }

    // set the information for the step being currently executed
    pub fn trace_prepare(&self, pc: usize, instruction: u8, gas_cost: U256) {
        self.curr_exec.borrow_mut().last_inst = instruction;
        self.curr_exec.borrow_mut().pc = pc;
        self.curr_exec.borrow_mut().gas_cost = gas_cost;
    }

    pub fn trace_add_instruction(
        &self, 
        gas_used: U256, 
        stack_push: &[U256], 
        mem_diff: Option<(usize, &[u8])>,
        store_diff: Option<(U256, U256)>
    ) {

        let mem_diff = match mem_diff {
            Some(v) => {
                Some((v.0, v.1.to_vec()))
            },
            None => None
        };
        let stack_push = stack_push.to_vec();
        self.inst_hist.borrow_mut().push(
            InstructionState {
                instruction: Instruction::from_u8(self.curr_exec.borrow().last_inst).unwrap(),
                pc: self.curr_exec.borrow().pc,
                gas_cost: self.curr_exec.borrow().gas_cost,
                stack_push, mem_diff, store_diff,
            }
        ); 
    }

    pub fn get_curr_pc(&self) -> usize {
        self.curr_exec.borrow().pc
    }
    
    /// resets struct
    pub fn reset(&self) {
        self.inst_hist.borrow_mut().clear();
        self.curr_exec.borrow_mut().clear();
    }
}

/*
impl<'a> Default for &'a InstructionManager {
    fn default() -> InstructionManager {
        InstructionManager {
            inst_hist: RefCell::new(Vec::new()),
            last_inst: 0,
            pc: 0,
            gas_cost: U256::zero(),
        }
    }

}

*/
