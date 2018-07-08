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
use evm::{CostType};
use evm::interpreter::{Interpreter, SharedCache, InterpreterResult};
use vm::{ActionParams};
use vm::{Vm, GasLeft};
use std::sync::Arc;
use std::rc::Rc;
use instruction_manager::InstructionManager;


/// A wrapper around Parity's Evm Interpreter implementation
pub struct InterpreterExt<'a, Cost: CostType> {
    interpreter: Interpreter<Cost>,
    cache: Arc<SharedCache>,
    params: ActionParams,
    inst_manager: &'a InstructionManager,
}

impl<'a, Cost: CostType> InterpreterExt<'a, Cost> {
    
    pub fn new(params: ActionParams, cache: Arc<SharedCache>, ext: &vm::Ext, inst_manager: &'a InstructionManager)
        -> vm::Result<InterpreterExt<'a, Cost>> 
    {

        Ok(InterpreterExt {
            params: params.clone(),
            cache: cache.clone(),
            interpreter: Interpreter::new(params, cache, ext).unwrap(),
            inst_manager,
        })
    }

    /// runs code without stopping at any position
    // pass through for vm::Vm exec
    pub fn run_code(&mut self, ext: &mut vm::Ext) -> vm::Result<GasLeft> {
        self.interpreter.exec(ext)
    }
    
    /// go back in execution to a position (PC). pos specifies how many
    /// VM steps to go back
    // actually just restarts vm until a pos
    // the most inefficient function so far
    pub fn step_back(&mut self, pos: usize, ext: &mut vm::Ext) {
        // Might be an issue, if cache isn't really a cache and used as a 
        // reference in Parity somewhere
        self.interpreter = Interpreter::new(
                                            self.params.clone(), 
                                            self.cache.clone(), 
                                            ext).unwrap();

        let steps = self.inst_manager.inst_hist.borrow();
        let new_pc = steps.get(steps.len() - pos);
        self.inst_manager.reset();
        self.run_code_until(ext, pos);
    }

    /// run code until an instruction
    /// stops before instruction execution (PC)
    pub fn run_code_until(&mut self, ext: &mut vm::Ext, pos: usize) 
        -> Option<vm::Result<GasLeft>>
    {
        while self.inst_manager.get_curr_pc() < pos {
            let result = self.interpreter.step(ext);
            match result {
                InterpreterResult::Continue => {},
                InterpreterResult::Done(value) => return Some(value),
                InterpreterResult::Stopped 
                    => panic!("Attempted to execute an already stopped VM.")
            }
        }
        None
    }
}

// some tests taken from ethcore::evm::tests
#[cfg(test)]
mod tests {
    use ethereum_types::{U256, H256, Address};
    use rustc_hex::FromHex;
    use tests::fake_ext::{FakeExt, test_finalize};
    use vm::{ActionParams};
    use std::sync::Arc;
    use evm::interpreter::{SharedCache};
    use std::str::FromStr;
    use instruction_manager::InstructionManager;
    use std::rc::Rc;

    
    #[test]
    fn it_should_run_code() {
        let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	    let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055".from_hex().unwrap();

        let mut params = ActionParams::default();
        params.address = address.clone();
        params.gas = U256::from(100_000);
        params.code = Some(Arc::new(code));
        let cache = Arc::new(SharedCache::default());
        let inst_manager = InstructionManager::new();
        let mut ext = FakeExt::new(&inst_manager);

        
        let gas_left = {
            let mut vm = super::InterpreterExt::<usize>::new(params, 
                                                             cache.clone(), 
                                                             &ext, 
                                                             &inst_manager).unwrap();
            test_finalize(vm.run_code(&mut ext)).unwrap()
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
        let inst_manager = InstructionManager::new();
        let mut ext = FakeExt::new(&inst_manager);
 
        let gas_left = {
            let mut vm = super::InterpreterExt::<usize>::new(params, 
                                                             cache.clone(), 
                                                             &ext, 
                                                             &inst_manager).unwrap();
            test_finalize(vm.run_code(&mut ext)).unwrap()
        };
    }
    
    // need trace to test further
    #[test]
    fn it_should_stop_after_ins() {
        let address = 
            Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();

        let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055".from_hex().unwrap();

        let mut params = ActionParams::default();
        params.address = address.clone();
        params.gas = U256::from(100_000);
        params.code = Some(Arc::new(code));
        let cache = Arc::new(SharedCache::default());
        let mut inst_manager = InstructionManager::new();
        let mut ext = FakeExt::new(&inst_manager);

        let mut vm = super::InterpreterExt::<usize>::new(params, 
                                                        cache.clone(), 
                                                        &ext, &inst_manager).unwrap();

        let gas_left = vm.run_code_until(&mut ext, 2);
        assert!(gas_left.is_none());
        println!("Instruction Manager: {:?}", vm.inst_manager);
        assert!(vm.inst_manager.get_curr_pc() >= 2);
        // println!("VM Step: {}", vm.);
    }

    #[test]
    fn it_should_exec_simple_contract() {
        let address = 
            Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();

        let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055".from_hex().unwrap();

        let mut params = ActionParams::default();
        params.address = address.clone();
        params.gas = U256::from(100_000);
        params.code = Some(Arc::new(code));
        let cache = Arc::new(SharedCache::default());
        let mut inst_manager = InstructionManager::new();
        let mut ext = FakeExt::new(&inst_manager);

        let mut vm = super::InterpreterExt::<usize>::new(params, 
                                                        cache.clone(), 
                                                        &ext, &inst_manager).unwrap();
        let gas_left = test_finalize(vm.run_code(&mut ext)).unwrap();
    }
}


