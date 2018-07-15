// Copyright 2015-2018 Andrew Plaza (U.S.A)
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
use debug_externalities::{ExternalitiesExt, DebugExt};
use ethereum_types::U256;
use evm::interpreter::{Interpreter, InterpreterResult};
use evm::interpreter::stack::VecStack;
use evm::{CostType};
use vm::{GasLeft, Vm};
use std::any::Any;
use std::marker::Send;

pub trait InterpreterExt {
    fn step_back(self, ext: &mut ExternalitiesExt) -> vm::Result<ExecInfo>;
    fn run_code_until(&mut self, ext: &mut ExternalitiesExt, pos: usize)
        -> vm::Result<ExecInfo>;
    fn run(&mut self, ext: &mut vm::Ext) -> vm::Result<ExecInfo>;
    fn get_curr_pc(&self) -> usize;
    fn as_any(&self) -> Box<Any + Send>;
}

pub trait AsInterpreter<C: CostType + Send> {
    fn as_interpreter(self) -> Option<Interpreter<C>>;
}
// this might be a very bad idea
// unsafe impl Send for InterpreterExt {}
// TODO change from returning Option to Result, for error handling
impl<C> AsInterpreter<C> for Box<Any + Send> 
    where C: CostType + Send + 'static,
{
    fn as_interpreter(self) -> Option<Interpreter<C>> {
        if let Ok(interpreter) = self.downcast::<Interpreter<C>>() {
            Some(*interpreter)
        } else { None }
    }
}

impl<C> InterpreterExt for Interpreter<C> where C: CostType + Send + 'static {

    /// go back one step in execution
    fn step_back(mut self, ext: &mut ExternalitiesExt) -> vm::Result<ExecInfo>{
        self = ext.step_back().as_any().as_interpreter().unwrap();
        Ok(ExecInfo::from_vm(&self, None))
    }

    /// run code until an instruction
    /// stops before instruction execution (PC)
    fn run_code_until(&mut self, ext: &mut ExternalitiesExt, pos: usize)-> vm::Result<ExecInfo> {   
        if ext.snapshots_len() <= 0 {
            ext.push_snapshot(Box::new(self.clone())); // empty state
        }
        while (self.reader.position) < pos {
            let result = self.step(ext.externalities());
            ext.push_snapshot(Box::new(self.clone()));
            match result {
                InterpreterResult::Continue => {},
                InterpreterResult::Done(value) => return Ok(ExecInfo::from_vm(&self, Some(value))),
                InterpreterResult::Stopped 
                    => panic!("Attempted to execute an already stopped VM.")
            }
        }
        Ok(ExecInfo::from_vm(&self, None))
    }

    /// passthrough for vm::Vm exec()
    fn run(&mut self, ext: &mut vm::Ext) -> vm::Result<ExecInfo> {
        let gas_left = self.exec(ext);
        Ok(ExecInfo::from_vm(&self, Some(gas_left)))
    }

    fn get_curr_pc(&self) -> usize {
        if self.reader.position <= 0 { self.reader.position}
        else { self.reader.position - 1 }
    }
    
    fn as_any(&self) -> Box<Any + Send> {
        Box::new(self.clone())
    }
}


#[derive(Debug, Clone)]
pub struct ExecInfo {
    mem: Vec<u8>,
    stack: VecStack<U256>,
    pc: usize,
    finished: bool,
    gas_left: Option<vm::Result<GasLeft>>
}

impl ExecInfo {
    pub fn new(mem: Vec<u8>, 
               stack: VecStack<U256>, 
               pc: usize, 
               gas_left: Option<vm::Result<GasLeft>>
    ) -> Self {
        ExecInfo {mem, stack, pc, gas_left, finished: false}
    }

    pub fn from_vm<C: CostType + Send + 'static>(interpreter: &Interpreter<C>, gas_left: Option<vm::Result<GasLeft>>
    ) -> Self {
        ExecInfo {
            mem: interpreter.mem.clone(),
            stack: interpreter.stack.clone(),
            pc: interpreter.get_curr_pc(),
            finished: if gas_left.is_none() {false} else {true},
            gas_left,
       }
    }

    pub fn empty(gas_left: Option<vm::Result<GasLeft>>) -> Self {
        ExecInfo {
            mem: Vec::default(),
            stack: VecStack::with_capacity(0usize, U256::zero()),
            pc: 0,
            finished: true,
            gas_left,
        }
    }

    pub fn mem(&self) -> &Vec<u8> {&self.mem}
    pub fn stack(&self) -> &VecStack<U256>{&self.stack}
    pub fn gas_left(&self) -> &Option<vm::Result<GasLeft>> {&self.gas_left}
    pub fn pc(&self) -> &usize {&self.pc}
    pub fn finished(&self) -> bool {self.finished}
}

// some tests taken from ethcore::evm::tests
#[cfg(test)]
mod tests {
    use super::InterpreterExt;
    use vm::{Vm, ActionParams};
    use ethereum_types::{U256, H256, Address};
    use rustc_hex::FromHex;
    use tests::fake_ext::{FakeExt, test_finalize};
    use std::sync::Arc;
    use evm::interpreter::{SharedCache, Interpreter};
    use std::str::FromStr;
    use instruction_manager::InstructionManager;
    use emulator::InterpreterSnapshots;
    use std::rc::Rc;

    
    #[test]
    fn it_should_run_trait_functions() {
        let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	    let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055".from_hex().unwrap();

        let mut params = ActionParams::default();
        params.address = address.clone();
        params.gas = U256::from(100_000);
        params.code = Some(Arc::new(code));
        let cache = Arc::new(SharedCache::default());
        let mut ext = FakeExt::new();

        
        let gas_left = {
            let mut vm = Interpreter::<usize>::new(params, cache.clone(), &ext).unwrap();
            test_finalize(vm.exec(&mut ext)).unwrap()
        };

        assert_eq!(gas_left, U256::from(79_988));
    //    assert_store(&ext, 0, "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe");
    }
    
    // just random code
    // contains bad instruction
    // this code segment becomes important in InstructionManager and Emulator
    #[test]
    #[should_panic]
    fn it_should_and_panic() {
        let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
        let code = "60806040526004361061006d576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff16806360fe47b1146100725780636d4ce63c1461009f5780639fc8192c146100ca578063c2d2c2ea146100f7578063dffeadd014610122575b600080fd5b34801561007e57600080fd5b5061009d60048036038101908080359060200190929190505050610139565b005b3480156100ab57600080fd5b506100b461014d565b6040518082815260200191505060405180910390f35b3480156100d657600080fd5b506100f560048036038101908080359060200190929190505050610156565b005b34801561010357600080fd5b5061010c610179565b6040518082815260200191505060405180910390f35b34801561012e57600080fd5b50610137610183565b005b806000819055506001810160018190555050565b60008054905090565b80600281905550600a600254016002819055506000546002540360028190555050565b6000600154905090565b61018d6014610139565b6101976032610156565b5600a165627a7a7230582073220057da31267f028c5802e52e8b0f18aac96f30d1dcc4cc9c9d2cfe5b28d40029".from_hex().unwrap();

        let mut params = ActionParams::default();
        params.address = address.clone();
        params.gas = U256::from(100_000_000);
        params.code = Some(Arc::new(code));
        let cache = Arc::new(SharedCache::default());
        let mut ext = FakeExt::new();
 
        let gas_left = {
            let mut vm = Interpreter::<usize>::new(params, cache.clone(), &ext).unwrap();
            test_finalize(vm.exec(&mut ext)).unwrap()
        };
    }
    
    // need trace to test further
    #[test]
    fn it_should_stop_after_ins() {
        let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();

        let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055".from_hex().unwrap();

        let mut params = ActionParams::default();
        params.address = address.clone();
        params.gas = U256::from(100_000);
        params.code = Some(Arc::new(code));
        let cache = Arc::new(SharedCache::default());
        let mut ext = FakeExt::new();
        let mut vm = Interpreter::<usize>::new(params, cache.clone(), &ext).unwrap();
        let mut i_hist = InterpreterSnapshots::new();
        
        let gas_left = vm.run_code_until(&mut ext, 2, &mut i_hist);
        assert!(gas_left.is_none());
        assert!(vm.get_curr_pc() >= 2);
        println!("VM Program Counter: {}", vm.get_curr_pc());
    }

    #[test]
    fn it_should_step_back() {
        let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
        let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055"
                    .from_hex().unwrap();
        let mut params = ActionParams::default();
        params.address = address.clone();
        params.gas = U256::from(100_000);
        params.code = Some(Arc::new(code));
        let cache = Arc::new(SharedCache::default());
        let mut ext = FakeExt::new();
        let mut vm = Interpreter::<usize>::new(params, cache.clone(), &ext).unwrap();
        let mut i_hist = InterpreterSnapshots::new();
        
        let gas_left = vm.run_code_until(&mut ext, 2, &mut i_hist);
        match gas_left {
            Some(x) => panic!("Execution should not have finished"),
            None => {
                println!("Program Counter before stepping back: {}", vm.get_curr_pc());
                vm = vm.step_back(&mut i_hist);
                println!("Program Counter after stepping back: {}", vm.get_curr_pc());
            }
        }
    }

    #[test]
    fn it_should_exec_simple_contract() {
        let address = 
            Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();

        let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055"
            .from_hex().unwrap();

        let mut params = ActionParams::default();
        params.address = address.clone();
        params.gas = U256::from(100_000);
        params.code = Some(Arc::new(code));
        let cache = Arc::new(SharedCache::default());
        let mut ext = FakeExt::new();
        let mut vm = super::Interpreter::<usize>::new(params, cache.clone(), &ext).unwrap();
        let gas_left = test_finalize(vm.exec(&mut ext)).unwrap();
    }
}


