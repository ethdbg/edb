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
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.


//! Fake Ext
//! Taken straight from Parity; modified to add functions which communicate with 
//! the InstructionManager in trace_ method calls
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

use ethereum_types::{U256, H256, Address};
use bytes::Bytes;
use vm::{
	CallType, Schedule, EnvInfo,
	ReturnData, Ext, ContractCreateResult, MessageCallResult,
	CreateContractAddress, Result, GasLeft,
};
use instruction_manager::InstructionManager;

pub struct FakeLogEntry {
	pub topics: Vec<H256>,
	pub data: Bytes
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum FakeCallType {
	Call, Create
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct FakeCall {
	pub call_type: FakeCallType,
	pub gas: U256,
	pub sender_address: Option<Address>,
	pub receive_address: Option<Address>,
	pub value: Option<U256>,
	pub data: Bytes,
	pub code_address: Option<Address>,
}

/// Fake externalities test structure.
///
/// Can't do recursive calls.
#[derive(Default)]
pub struct FakeExt {
	pub store: HashMap<H256, H256>,
	pub suicides: HashSet<Address>,
	pub calls: HashSet<FakeCall>,
	pub sstore_clears: usize,
	pub depth: usize,
	pub blockhashes: HashMap<U256, H256>,
	pub codes: HashMap<Address, Arc<Bytes>>,
	pub logs: Vec<FakeLogEntry>,
	pub info: EnvInfo,
	pub schedule: Schedule,
	pub balances: HashMap<Address, U256>,
	pub tracing: bool,
	pub is_static: bool,
        // pub inst_manager: &'a InstructionManager,
}

// similar to the normal `finalize` function, but ignoring NeedsReturn.
pub fn test_finalize(res: Result<GasLeft>) -> Result<U256> {
	match res {
		Ok(GasLeft::Known(gas)) => Ok(gas),
		Ok(GasLeft::NeedsReturn{..}) => unimplemented!(), // since ret is unimplemented.
		Err(e) => Err(e),
	}
}

impl FakeExt {
	/// New fake externalities
	pub fn new() -> Self {
        FakeExt {
            store: HashMap::default(),
            suicides: HashSet::default(),
            calls: HashSet::default(),
            sstore_clears: 0,
            depth: 0,
            blockhashes: HashMap::default(),
            codes: HashMap::default(),
            logs: Vec::default(),
            info: EnvInfo::default(),
            schedule: Schedule::default(),
            balances: HashMap::default(),
            tracing: true,
            is_static: false,
            // inst_manager,
        }
	}

	/// New fake externalities with byzantium schedule rules
	pub fn new_byzantium() -> Self {
		let mut ext = FakeExt::default();
		ext.schedule = Schedule::new_byzantium();
		ext
	}

	/// New fake externalities with constantinople schedule rules
	pub fn new_constantinople() -> Self {
		let mut ext = FakeExt::default();
		ext.schedule = Schedule::new_constantinople();
		ext
	}

	/// Alter fake externalities to allow wasm
	pub fn with_wasm(mut self) -> Self {
		self.schedule.wasm = Some(Default::default());
		self
	}
}

impl Ext for FakeExt {
	fn storage_at(&self, key: &H256) -> Result<H256> {
		Ok(self.store.get(key).unwrap_or(&H256::new()).clone())
	}

	fn set_storage(&mut self, key: H256, value: H256) -> Result<()> {
		self.store.insert(key, value);
		Ok(())
	}

	fn exists(&self, address: &Address) -> Result<bool> {
		Ok(self.balances.contains_key(address))
	}

	fn exists_and_not_null(&self, address: &Address) -> Result<bool> {
		Ok(self.balances.get(address).map_or(false, |b| !b.is_zero()))
	}

	fn origin_balance(&self) -> Result<U256> {
		unimplemented!()
	}

	fn balance(&self, address: &Address) -> Result<U256> {
		Ok(self.balances[address])
	}

	fn blockhash(&mut self, number: &U256) -> H256 {
		self.blockhashes.get(number).unwrap_or(&H256::new()).clone()
	}

	fn create(&mut self, gas: &U256, value: &U256, code: &[u8], _address: CreateContractAddress) -> ContractCreateResult {
		self.calls.insert(FakeCall {
			call_type: FakeCallType::Create,
			gas: *gas,
			sender_address: None,
			receive_address: None,
			value: Some(*value),
			data: code.to_vec(),
			code_address: None
		});
		ContractCreateResult::Failed
	}

	fn call(&mut self,
			gas: &U256,
			sender_address: &Address,
			receive_address: &Address,
			value: Option<U256>,
			data: &[u8],
			code_address: &Address,
			_output: &mut [u8],
			_call_type: CallType
		) -> MessageCallResult {

		self.calls.insert(FakeCall {
			call_type: FakeCallType::Call,
			gas: *gas,
			sender_address: Some(sender_address.clone()),
			receive_address: Some(receive_address.clone()),
			value: value,
			data: data.to_vec(),
			code_address: Some(code_address.clone())
		});
		MessageCallResult::Success(*gas, ReturnData::empty())
	}

	fn extcode(&self, address: &Address) -> Result<Arc<Bytes>> {
		Ok(self.codes.get(address).unwrap_or(&Arc::new(Bytes::new())).clone())
	}

	fn extcodesize(&self, address: &Address) -> Result<usize> {
		Ok(self.codes.get(address).map_or(0, |c| c.len()))
	}

	fn log(&mut self, topics: Vec<H256>, data: &[u8]) -> Result<()> {
		self.logs.push(FakeLogEntry {
			topics: topics,
			data: data.to_vec()
		});
		Ok(())
	}

	fn ret(self, _gas: &U256, _data: &ReturnData, _apply_state: bool) -> Result<U256> {
        unimplemented!();
    }

	fn suicide(&mut self, refund_address: &Address) -> Result<()> {
		self.suicides.insert(refund_address.clone());
		Ok(())
	}

	fn schedule(&self) -> &Schedule {
		&self.schedule
	}

	fn env_info(&self) -> &EnvInfo {
		&self.info
	}

	fn depth(&self) -> usize {
		self.depth
	}

	fn is_static(&self) -> bool {
		self.is_static
	}

	fn inc_sstore_clears(&mut self) {
		self.sstore_clears += 1;
	}

	fn trace_next_instruction(&mut self, _pc: usize, _instruction: u8, _gas: U256) -> bool {
		self.tracing
	}

    /// Prepare to trace an operation. Passthrough for the VM trace
    fn trace_prepare_execute(
        &mut self, 
        pc: usize, 
        instruction: u8, 
        gas_cost: U256) 
    {   
        //self.inst_manager.trace_prepare(pc, instruction, gas_cost);
    }

    /// Trace the finalised execution of a single instruction
    fn trace_executed(
        &mut self, 
        gas_used: U256, 
        stack_push: &[U256], 
        mem_diff: Option<(usize, &[u8])>, 
        store_diff: Option<(U256, U256)>) 
    {   
        // self.inst_manager.trace_add_instruction(gas_used, stack_push, mem_diff, store_diff);
    }
}


