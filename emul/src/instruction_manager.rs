//! Manages Instructions/Traces 
use ethereum_types::{U256, H256, Address};
use evm::{Instruction};

struct InstructionState {
    instruction: Instruction,
    pc: usize,
    gas_cost: U256,
    stack_push: Vec<U256>,
    mem_diff: Option<(usize, Vec<u8>)>,
    store_diff: Option<(U256, U256)>,
}

pub struct InstructionManager {
    instruction_hist: Vec<InstructionState>,
    last_inst: u8,
    pc: usize,
    gas_cost: U256,
}

impl InstructionManager {

    pub fn new() -> InstructionManager {
        InstructionManager {
            instruction_hist: Vec::new(),
            last_inst: 0,
            pc: 0,
            gas_cost: U256::zero(),
        }
    }

    pub fn trace_prepare(&mut self, pc: usize, instruction: u8, gas_cost: U256) {
        self.pc = pc;
        self.last_inst = instruction;
        self.gas_cost = gas_cost;
    }


    pub fn trace_add_instruction(
        &mut self, 
        gas_used: U256, 
        stack_push: &[U256], 
        mem_diff: Option<(usize, &[u8])>,
        store_diff: Option<(U256, U256)>
    ) {
        let mem_diff = mem_diff.unwrap();
        let stack_push = stack_push.to_vec();
        self.instruction_hist.push(
            InstructionState {
                instruction: Instruction::from_u8(self.last_inst).unwrap(),
                pc: self.pc,
                gas_cost: self.gas_cost,
                stack_push,
                mem_diff: Some((mem_diff.0, mem_diff.1.to_vec())),
                store_diff,
            }); 
    }
}


