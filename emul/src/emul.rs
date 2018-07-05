// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with EDB. If not, see <http://www.gnu.org/licenses/>.

use vm;
use ethereum_types::{U256, H256, Address};
use evm::{VMType, FinalizationResult, CostType};
use evm::factory::Factory;
use evm::interpreter::{Interpreter, SharedCache};
use vm::{ActionParams, CallType, ActionValue};
use vm::{Vm, Ext, GasLeft};
use vm::tests::FakeExt;
use hash::{keccak, KECCAK_EMPTY};
use bytes::Bytes;
use std::sync::Arc;

/// A wrapper around Parity's Evm implementation
pub struct Emul {
    params: ActionParams,
//    vm: Interpreter<Cost>,
}

impl Emul {
    pub fn new(params: ActionParams) -> Result<Emul, &'static str> {
        let mut params = ActionParams::default();
 

        Ok(Emul {
            params,
    //      vm
        })
    }

    pub fn run_code(&self) -> vm::GasLeft {
        let mut ext = FakeExt::new();
        let mut vm: Box<Vm> = 
            Factory::new(VMType::Interpreter, 10 * 1024)
            .create(self.params.clone(), &ext).unwrap();
        ext.balances.insert(5.into(), 1_000_000_000.into());
        ext.tracing = true;
        let output = vm.exec(&mut ext).unwrap();
        output
    }

    pub fn next_inst() {
        unimplemented!(); 
    }
}

#[derive(Clone, Debug)]
pub struct Args {
	/// Address of currently executed code.
	pub to: Option<Address>,
    /// from address
    pub from: Option<Address>,
	/// Hash of currently executed code.
	pub code_hash: Option<H256>,
	/// Gas paid up front for transaction execution
	pub gas: Option<U256>,
	/// Gas price.
	pub gas_price: Option<U256>,
	/// Code being executed.
	pub code: Option<Bytes>,
	/// Input data.
	pub data: Option<Bytes>,
}

// not sure what args are required yet
impl Args {
    pub fn new(to: Option<Address>, 
               from: Option<Address>, 
               code_hash: Option<H256>,
               gas: Option<U256>,
               gas_price: Option<U256>, 
               code: Option<Bytes>,
               data: Option<Bytes>
        ) -> Result<Args, &'static str> {

        Ok(Args {
            to,
            from,
            code_hash,
            gas,
            gas_price,
            code,
            data
        })
    }
}

