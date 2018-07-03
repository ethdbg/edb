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

use std::sync::Arc;
use std::cmp;
use std::collections::{HashMap, HashSet};
use ethcore::state::{Backend as StateBackend, State, Substate, CleanupMode};
use ethcore::trace::{Tracer, VMTracer};
use ethereum_types::{U256, H256, Address};
use ethcore::machine::EthereumMachine;
use bytes::{Bytes, BytesRef};
use evm::FinalizationResult;
use vm::{self, CallType, ActionParams, ActionValue, Schedule, EnvInfo, ReturnData, Ext, 
    ContractCreateResult, MessageCallResult, CreateContractAddress, 
    Result, GasLeft
};
use transaction::UNSIGNED_SENDER;
use executive::*;

/// Policy for handling output data on `RETURN` opcode.
pub enum OutputPolicy<'a, 'b> {
    /// Return reference to fixed sized output.
    /// Used for message calls
    Return(BytesRef<'a>, Option<&'b mut Bytes>),
    InitContract(Option<&'b mut Bytes>),
}

pub struct OriginInfo {
    address: Address,
    origin: Address,
    gas_price: U256,
    value: U256,
}

impl OriginInfo {
    pub fn from(params: &ActionParams) -> Self  {
        OriginInfo {
            address: params.address.clone(),
            origin: params.origin.clone(),
            gas_price: params.gas_price,
            value: match params.value {
                ActionValue::Transfer(val) | ActionValue::Apparent(val) => val
            },
        }
    }
}

// psuedo externalities for debugging
// may make work with testRPC in future
pub struct Externalities<'a, T: 'a, V: 'a, B: 'a> 
    where T: Tracer, V: VMTracer, B: StateBackend
{
    state: &'a mut State<B>,
    env_info: &'a EnvInfo,
    machine: &'a EthereumMachine,
    depth: usize,
    origin_info: OriginInfo,
    substate: &'a mut Substate,
    schedule: Schedule,
    output: OutputPolicy<'a, 'a>,
    tracer: &'a mut T,
    vm_tracer: &'a mut V,
    static_flag: bool,
}

impl<'a, T: 'a, V: 'a, B: 'a> Externalities<'a, T, V, B> 
    where T: Tracer, V: VMTracer, B: StateBackend
{
    pub fn new(state: &'a mut State<B>,
        env_info: &'a EnvInfo,
        machine: &'a EthereumMachine,
        depth: usize,
        origin_info: OriginInfo,
        substate: &'a mut Substate,
        output: OutputPolicy<'a, 'a>,
        tracer: &'a mut T,
        vm_tracer: &'a mut V,
        static_flag: bool
        ) -> Self {

            Externalities {
                state,
                env_info,
                machine,
                depth,
                origin_info,
                substate,
                schedule: machine.schedule(env_info.number),
                output,
                tracer,
                vm_tracer,
                static_flag,
            }
    }
}

impl<'a, T: 'a, V: 'a, B: 'a> Ext for Externalities<'a, T, V, B> 
    where T: Tracer, V: VMTracer, B: StateBackend
{

    /// Returns a value for a given key
    /// (Look: Eth Yellow Paper Account Storage DB)
    fn storage_at(&self, key: &H256) -> vm::Result<H256> {
           self.state.storage_at(&self.origin_info.address, key).map_err(Into::into)
    }
    
    /// Stores value for a given key
    fn set_storage(&mut self, key: H256, value: H256) -> Result<()> {
        if self.static_flag {
            Err(vm::Error::MutableCallInStaticContext)
        } else {
            self.state.set_storage(&self.origin_info.address, key, value).map_err(Into::into)
        }
    }
    /// Determine whether account exists
    fn exists(&self, address: &Address) -> vm::Result<bool> {
        self.state.exists(address).map_err(Into::into)
    }
    
    /// determine wheter account exists and is not null (zero balance/nonce, 
    /// no code)
    fn exists_and_not_null(&self, address: &Address) -> vm::Result<bool> {
        self.state.exists_and_not_null(address).map_err(Into::into)
    }
    
    /// balance of the origin account
    fn origin_balance(&self) -> vm::Result<U256> {
        self.balance(&self.origin_info.address).map_err(Into::into)
    }
    
    fn balance(&self, address: &Address) -> Result<U256> {
        self.state.balance(address).map_err(Into::into)
    }
    
    /// returns the hash of one of the 256 most recent complete blocks
    fn blockhash(&mut self, number: &U256) -> H256 {
        if self.env_info.number + 256 >= self.machine.params().eip210_transition {
            let blockhash_contract_address = 
                self.machine.params().eip210_contract_address;
            let code_res = self.state.code(&blockhash_contract_address)
                .and_then(
                    |code| self.state.code_hash(&blockhash_contract_address)
                          .map(|hash| (code, hash))
            );

            let (code, code_hash) = match code_res {
                Ok((code, hash)) => (code, hash),
                Err(_) => return H256::zero(),
            };

            let params = ActionParams {
                sender: self.origin_info.address.clone(),
                address: blockhash_contract_address.clone(),
                value: ActionValue::Apparent(self.origin_info.value),
                code_address: blockhash_contract_address.clone(),
                origin: self.origin_info.origin.clone(),
                gas: self.machine.params().eip210_contract_gas,
                gas_price: 0.into(),
                code: code,
                code_hash: Some(code_hash),
                data: Some(H256::from(number).to_vec()),
                call_type: CallType::Call,
                params_type: vm::ParamsType::Separate,
            };

            let mut output = H256::new();
            let mut ex = 
                Executive::new(self.state, self.env_info, self.machine);
            let r = ex.call(params, 
                        self.substate, 
                        BytesRef::Fixed(&mut output), 
                        self.tracer, 
                        self.vm_tracer);
            output
        } else {
            match *number < U256::from(self.env_info.number) && number.low_u64()
                >= cmp::max(256, self.env_info.number) - 256 
            {
                true => {
                    let index = self.env_info.number - number.low_u64() -1;
                    assert!(index < self.env_info.last_hashes.len() as u64, 
                            format!("Inconsistent env_info, \
                                    should contain at least {:?} \
                                    last hashes", index+1));
                    let r = 
                        self.env_info.last_hashes[index as usize].clone();
                    trace!(
                        "ext: blockhash({}) -> {} self.env_info.number={}\n", 
                        number, r, self.env_info.number
                    );
                    r
                },
                false => {
                    trace!(
                        "ext: blockhash({}) -> null self.env_info.number={}\n",
                        number, self.env_info.number
                    );
                    H256::zero()
                },
            }
        }
    }

    /// Creates a new Contract.
    ///
    /// Returns gas_left and contract address if contract creation was succesful
    fn create(&mut self, gas: &U256, value: &U256, code: &[u8], address_scheme: CreateContractAddress) -> ContractCreateResult {

        let (address, code_hash) = 
            match self.state.nonce(&self.origin_info.address) {
            
            Ok(nonce) => 
                contract_address(address_scheme, 
                                 &self.origin_info.address, 
                                 &nonce, 
                                 &code),
            Err(e) => {
                debug!(target: "ext", 
                       "Database corruption encountered: {:?}", e);
                return ContractCreateResult::Failed;
            }
        };
        
        let params = ActionParams {
            code_address: address.clone(),
            address: address.clone(),
            sender: self.origin_info.address.clone(),
            origin: self.origin_info.origin.clone(),
            gas: *gas,
            gas_price: self.origin_info.gas_price,
            value: ActionValue::Transfer(*value),
            code: Some(Arc::new(code.to_vec())),
            code_hash: code_hash,
            data: None,
            call_type: CallType::None,
            params_type: vm::ParamsType::Embedded,
        };
        
        if !self.static_flag {
            if !self.schedule.eip86 || params.sender != UNSIGNED_SENDER {
                if let Err(e) = self.state.inc_nonce(&self.origin_info.address) {
                    debug!(target: "ext", 
                           "Database corruption encountered: {:?}", e);
                    return ContractCreateResult::Failed;
                }
            }
        }
        let mut ex = Executive::from_parent(self.state, self.env_info, self.machine, self.depth, self.static_flag);
        match ex.create(params, 
                        self.substate, 
                        &mut None, 
                        self.tracer, 
                        self.vm_tracer
        ) {
            Ok(FinalizationResult{ gas_left, apply_state: true, .. }) => {
                self.substate.contracts_created.push(address.clone());
                ContractCreateResult::Created(address, gas_left)
            },
            Ok(FinalizationResult{ gas_left, apply_state: false, return_data }) => {
                ContractCreateResult::Reverted(gas_left, return_data)
            },
            _ => ContractCreateResult::Failed,
        }
    }

    /// Message call.
    ///
    /// Returns Err, if we run out of gas.
    /// Otherwise returns call_result which contains gas left
    /// and true if subcall was successfull.
    fn call(&mut self,
            gas: &U256,
            sender_address: &Address,
            receive_address: &Address,
            value: Option<U256>,
            data: &[u8],
            code_address: &Address,
            output: &mut [u8],
            call_type: CallType
            ) -> MessageCallResult 
    {
        trace!(target: "externalities", "call");
        let code_res = self.state.code(code_address)
            .and_then(|code| self.state.code_hash(code_address)
                .map(|hash| (code, hash)));
        let (code, code_hash) = match code_res {
            Ok((code, hash)) => (code, hash),
            Err(_) => return MessageCallResult::Failed,
        };

        let mut params = ActionParams {
            sender: sender_address.clone(),
            address: receive_address.clone(),
            value: ActionValue::Apparent(self. origin_info.value),
            code_address: code_address.clone(),
            origin: self.origin_info.origin.clone(),
            gas: *gas,
            gas_price: self.origin_info.gas_price,
            code: code,
            code_hash: Some(code_hash),
            data: Some(data.to_vec()),
            call_type: call_type,
            params_type: vm::ParamsType::Separate,
        };
        
        if let Some(value) = value {
            params.value = ActionValue::Transfer(value);
        }

        let mut executive = Executive::from_parent(self.state, 
                                            self.env_info, 
                                            self.machine, 
                                            self.depth, 
                                            self.static_flag);
        match executive.call(params, 
                      self.substate, 
                      BytesRef::Fixed(output), 
                      self.tracer, 
                      self.vm_tracer
        ) {
            Ok(FinalizationResult{ gas_left, return_data, apply_state: true }) 
                => MessageCallResult::Success(gas_left, return_data),
            Ok(FinalizationResult{ gas_left, return_data, apply_state: false }) 
                => MessageCallResult::Reverted(gas_left, return_data),
            _ => MessageCallResult::Failed,
        }
    }

    /// Returns code at given address
    fn extcode(&self, address: &Address) -> Result<Arc<Bytes>> {
        // gets the code at an address, or just returns an empty ref-counted
        // vecctor
        Ok(self.state.code(address)?.unwrap_or_else(|| Arc::new(vec![])))
    }

    /// Returns code size at a given address
    fn extcodesize(&self, address: &Address) -> Result<usize> {
        // gets code size at an address, or returns 0
        Ok(self.state.code_size(address)?.unwrap_or(0))
    }

    /// Creates log entry with given topics and data
    fn log(&mut self, topics: Vec<H256>, data: &[u8]) -> Result<()> {
        use ethcore::log_entry::LogEntry;

        if self.static_flag {
            return Err(vm::Error::MutableCallInStaticContext);
        }

        let address = self.origin_info.address.clone();
        self.substate.logs.push(LogEntry {
            address: address,
            topics: topics,
            data: data.to_vec()
        });
        Ok(())
    }
    
    /// Should be called when transaction calls the `RETURN` opcode.
    /// Returns gas_left if cost of returning the data is not too high.
    fn ret(mut self, gas: &U256, data: &ReturnData, apply_state: bool) 
        -> Result<U256> where Self: Sized 
    {
        let handle_copy = |to: &mut Option<&mut Bytes>| {
            to.as_mut().map(|b| **b = data.to_vec());
        };

        match self.output {
            OutputPolicy::Return(
                BytesRef::Fixed(ref mut slice), 
                ref mut copy) 
            => {
                handle_copy(copy);

                let len = cmp::min(slice.len(), data.len());
                (&mut slice[..len]).copy_from_slice(&data[..len]);
                Ok(*gas)
            }
            OutputPolicy::Return(
                BytesRef::Flexible(ref mut vec), 
                ref mut copy)
            => {
                handle_copy(copy);

                vec.clear();
                vec.extend_from_slice(&*data);
                Ok(*gas)
            },
            OutputPolicy::InitContract(ref mut copy) if apply_state => {
                let return_cost = 
                    U256::from(data.len()) * 
                        U256::from(self.schedule.create_data_gas);
                if return_cost > 
                    *gas || data.len() > 
                    self.schedule.create_data_limit
                {
                    return match self.schedule.exceptional_failed_code_deposit {
                        true => Err(vm::Error::OutOfGas),
                        false =>  Ok(*gas)
                    }
                }
                handle_copy(copy);
                self.state.init_code(&self.origin_info.address, data.to_vec())?;
                Ok(*gas - return_cost)
            },
            OutputPolicy::InitContract(_) => {
                Ok(*gas)
            },
        }
    }
    
    /// Should be called when contract commits suicide
    /// Address to which funds should be refunded.
    fn suicide(&mut self, refund_address: &Address) -> Result<()> {
        if self.static_flag {
            return Err(vm::Error::MutableCallInStaticContext);
        }

        let address = self.origin_info.address.clone();
        let balance = self.balance(&address)?;
        if &address == refund_address {
            // TODO to be consistent with CPP client we 
            // set balance to 0 in this case
            self.state.sub_balance(&address, &balance, &mut CleanupMode::NoEmpty)?;
        } else {
            trace!(target: "ext", 
                   "Suiciding {} -> {} (xfer: {})",
                   address,
                   refund_address,
                   balance);
            self.state.transfer_balance(
                &address,
                refund_address,
                &balance,
                self.substate.to_cleanup_mode(&self.schedule)
            )?;
        }
        self.tracer.trace_suicide(address, balance, refund_address.clone());
        self.substate.suicides.insert(address);
        Ok(())
    }
    
    /// returns schedule
    fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    /// Returns environment info
    fn env_info(&self) -> &EnvInfo {
        self.env_info
    }
    
    /// Returns current depth of execution
    ///
    /// If contract A calls contract B, and contract B calls C,
    /// then A depth is 0, B is 1, C is 2, and so on
    fn depth(&self) -> usize {
        self.depth
    }
    
    /// Increments sstore refunds count by 1
    fn inc_sstore_clears(&mut self) {
        self.substate.sstore_clears_count = 
            self.substate.sstore_clears_count + U256::one();
    }
    
    /// Decide if anymore operations should be traced. 
    /// Passthrough for the VMTrace
    fn trace_next_instruction(
        &mut self, 
        pc: usize, 
        instruction: u8, 
        current_gas: U256) -> bool 
    {
        self.vm_tracer.trace_next_instruction(pc, instruction, current_gas)
    }

    /// Prepare to trace an operation. Passthrough for the VM trace
    fn trace_prepare_execute(
        &mut self, 
        pc: usize, 
        instruction: u8, 
        gas_cost: U256) 
    {
        self.vm_tracer.trace_prepare_execute(pc, instruction, gas_cost)
    }

    /// Trace the finalised execution of a single instruction
    fn trace_executed(
        &mut self, 
        gas_used: U256, 
        stack_push: &[U256], 
        mem_diff: Option<(usize, &[u8])>, 
        store_diff: Option<(U256, U256)>) 
    {
        self.vm_tracer.trace_executed(gas_used, stack_push, mem_diff, store_diff)
    }

    /// check if running in static context
    fn is_static(&self) -> bool {
        self.static_flag
    }
}
