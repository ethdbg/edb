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
pub struct InterpreterExt<Cost: CostType> {
    interpreter: Interpreter<Cost>,
    cache: Arc<SharedCache>,
    params: ActionParams,
    pub pos: usize,
    InstructionHistory: Vec<InstructionState>,
}

struct InstructionState {
    instruction: Instruction,
    mem_diff: Option<(usize, Vec<u8>)>,
    stack_push: Vec<u8>,
    store_diff: Option<(U256, U256)>,
}

impl<Cost: CostType> InterpreterExt<Cost> {
    
    pub fn new(params: ActionParams, cache: Arc<SharedCache>, ext: &vm::Ext)
        -> vm::Result<InterpreterExt<Cost>> 
    {
        Ok(InterpreterExt {
            params: params.clone(),
            cache: cache.clone(),
            interpreter: Interpreter::new(params, cache, ext).unwrap(),
            pos: 0,
            InstructionHistory: Vec::new(),
        })
    }

    /// runs code without stopping at any position
    // pass through for vm::Vm exec
    pub fn run_code(&mut self, ext: &mut vm::Ext) -> vm::Result<GasLeft> {
        self.interpreter.exec(ext)
    }
    
    /// go back in execution to a position
    // actually just restarts vm until a pos
    // the most inefficient function so far
    pub fn step_back(&mut self, pos: usize, ext: &mut vm::Ext) {
        // Might be an issue, if cache isn't really a cache and used as a 
        // reference in Parity somewhere
        self.interpreter = Interpreter::new(self.params.clone(), self.cache.clone(), ext).unwrap();
        let new_pos: usize = self.pos - pos;
        self.pos = 0;
        self.run_code_until(ext, new_pos);
    }

    /// run code until a byte position
    /// stops before byte position
    pub fn run_code_until(&mut self, ext: &mut vm::Ext, pos: usize) 
        -> Option<vm::Result<GasLeft>>
    {
        while self.pos < pos {
            let result = self.interpreter.step(ext);
            match result {
                InterpreterResult::Continue => {},
                InterpreterResult::Done(value) => return Some(value),
                InterpreterResult::Stopped 
                    => panic!("Attempted to execute an already stopped VM.")
            }
            self.pos += 1;
        }
        None
    }
}



