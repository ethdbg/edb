// Copyright 2015-2018 Andrew Plaa (U.S.A) Ltd.
// This file is part of EDB.
//
// EDB is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// EDB is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with EDB. If not, see <http://www.gnu.org/licenses/>.


//! An Extension to the parity interpreter for debugging 

use vm;
use ethereum_types::{U256, H256, Address};
use evm::{VMType, FinalizationResult, CostType, Instruction};
use evm::interpreter::{Interpreter, SharedCache, InterpreterResult};
use vm::{ActionParams, CallType, ActionValue};
use vm::{Vm, Ext, GasLeft};
use vm::tests::{FakeExt};
use std::sync::Arc;
use factory::Factory;


/// A wrapper around Parity's Evm Interpreter implementation
pub struct<Cost: CostType> InterpreterExt<Cost> {
    params: ActionParams,
    pos: usize,
    InstructionHistory: Vec<InstructionState>,
}

struct InstructionState {
    instruction: Instruction,
    mem_diff: Option<(usize, Vec<u8>)>,
    stack_push: Vec<u8>,
    store_diff: Option<(U256, U256)>,
}

pub trait InterpreterExt {
    fn new(params: ActionParams) -> Result<Emulator, &'static str>;

    fn run_code(&self, &mut vm::Ext) -> vm::Result<GasLeft>;
    
    fn step_back(&self, pos: usize);

    fn run_code_until(&self, ext: &mut vm::Ext, pos: usize);
}


impl<Cost: CostType> InterpreterExt for Interpreter<Cost> {
    pub fn new(params: ActionParams) -> Result<Emulator, &'static str> {

        Ok(Emulator {
            params,
            pos: 0,
            InstructionHistory: Vec::new()
        })
    }
    
    /// runs code without stopping at any position
    // pass through for vm::Vm exec
    pub fn run_code(&mut self, ext: &mut vm::Ext) -> vm::Result<GasLeft> {
        self.exec(ext)
    }
    
    /// go back in execution to a position
    // actually just restarts vm until a pos
    pub fn step_back(&self, pos: usize) {
        let new_pos = self.pos - pos;
        
    }

    /// run code until a byte position
    /// stops before byte position
    pub fn run_code_until(&self, ext: &mut vm::Ext, pos: usize) {

        while self.pos < pos {
            let result = self.step(ext);
            match result {
                InterpreterResult::Continue => {},
                InterpreterResult::Done(value) => return value,
                InterpreterResult::Stopped 
                    => panic!("Attempted to execute an already stopped VM.")
            }
            self.pos += 1;
        }
    }
}



