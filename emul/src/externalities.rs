// macros
use delegate::*;
use ethcore::externalities::{Externalities, OriginInfo, OutputPolicy};
use ethcore::state::{Backend as StateBackend, State, Substate};
use ethcore::machine::EthereumMachine as Machine;
use ethcore::trace::{Tracer, VMTracer};
use vm::{EnvInfo, Schedule, Ext, CreateContractAddress, CallType, MessageCallResult, ContractCreateResult, ReturnData};
use bytes::Bytes;
use std::sync::Arc;
use ethereum_types::{H256, U256, Address};
use crate::emulator::InterpreterSnapshots;
use crate::extensions::InterpreterExt;

//TODO move debug_externalities to extensions under externalities_ext;
//will require refactoring of use's

pub struct DebugExt<'a, T: 'a, V: 'a, B: 'a> {
    pub externalities: Externalities<'a, T, V, B>,
    snapshots: InterpreterSnapshots,
}

pub trait ExternalitiesExt: Ext {
    fn push_snapshot(&mut self, interpreter: Box<dyn InterpreterExt + Send>);
    fn step_back(&mut self) -> Box<dyn InterpreterExt + Send>;
    fn snapshots_len(&self) -> usize;
    fn externalities(&mut self) -> &mut dyn vm::Ext;
    // fn consume_ext(self) -> vm::Ext;
}

pub trait ConsumeExt<'a, T: 'a, V: 'a, B: 'a> {
    fn consume(self) -> Externalities<'a, T, V, B>
        where T: Tracer,
              V: VMTracer,
              B: StateBackend;
}

impl<'a, T: 'a, V: 'a, B: 'a> ConsumeExt<'a, T, V, B> for Externalities<'a, T, V, B> {
    fn consume(self) -> Externalities<'a, T, V, B>
        where   T: Tracer, 
                V: VMTracer, 
                B: StateBackend, 
    {
        self
    }
}

impl<'a, T: 'a, V: 'a, B: 'a> ConsumeExt<'a, T, V, B> for DebugExt<'a, T, V, B> {
    fn consume(self) -> Externalities<'a, T, V, B>
        where T: Tracer,
              V: VMTracer,
              B: StateBackend,
    {
        self.externalities
    }
}

impl<'a, T: 'a, V: 'a, B: 'a> DebugExt<'a, T, V, B> 
    where T: Tracer,
          V: VMTracer,
          B: StateBackend,
{   
    pub fn new( state: &'a mut  State<B>,
                env_info: &'a EnvInfo,
                machine: &'a Machine,
                schedule: &'a Schedule,
                depth: usize,
                origin_info: OriginInfo,
                substate: &'a mut Substate,
                output: OutputPolicy,
                tracer: &'a mut T,
                vm_tracer: &'a mut V,
                static_flag: bool
    ) -> Self {
        DebugExt {
            externalities: Externalities::new(state, env_info, machine, schedule, depth, origin_info, 
                                              substate, output, tracer, vm_tracer, static_flag),
            snapshots: InterpreterSnapshots::new()
        }
    }
}

impl<'a, T: 'a, V: 'a, B: 'a> Ext for DebugExt<'a, T, V, B> 
    where T: Tracer, V: VMTracer, B: StateBackend 
{
    delegate! {
        target self.externalities {
            fn storage_at(&self, key: &H256) -> vm::Result<H256>;
            
            fn set_storage(&mut self, key: H256, value:  H256) -> vm::Result<()>;

            fn exists(&self, address: &Address) -> vm::Result<bool>;

            fn exists_and_not_null(&self, address: &Address) -> vm::Result<bool>;

            fn origin_balance(&self) -> vm::Result<U256>;

            fn balance(&self, address: &Address) -> vm::Result<U256>;

            fn blockhash(&mut self, number: &U256) -> H256;

            fn create(&mut self, gas: &U256, value: &U256, code: &[u8], 
                      address: CreateContractAddress) -> ContractCreateResult;

            fn call(&mut self, 
                    gas: &U256, 
                    sender_address: &Address, 
                    receive_address: &Address, 
                    value: Option<U256>, 
                    data: &[u8], 
                    code_address: &Address, 
                    call_type: CallType
            ) -> MessageCallResult;

            fn extcode(&self, address: &Address) -> vm::Result<Arc<Bytes>>;

            fn extcodesize(&self, address: &Address) -> vm::Result<usize>;

            fn log(&mut self, topics: Vec<H256>, data: &[u8]) -> vm::Result<()>;

            fn ret(self, gas: &U256, data: &ReturnData, apply_state: bool) -> vm::Result<U256>;

            fn suicide(&mut self, refund_address: &Address) -> vm::Result<()>;

            fn schedule(&self) -> &Schedule;

            fn env_info(&self) -> &EnvInfo;

            fn depth(&self) -> usize;

            fn inc_sstore_clears(&mut self);

            fn trace_next_instruction(&mut self, 
                                      pc: usize, 
                                      instruction: u8, 
                                      current_gas: U256
            ) -> bool;

            fn trace_prepare_execute(&mut self, pc: usize, instruction: u8, gas_cost: U256);

            fn trace_executed(&mut self, 
                              gas_used: U256, 
                              stack_push: &[U256], 
                              mem_diff: Option<(usize, &[u8])>, 
                              store_diff: Option<(U256, U256)>
            );

            fn is_static(&self) -> bool;
        }
    }
}

impl<'a, T: 'a, V: 'a, B: 'a> ExternalitiesExt for DebugExt<'a, T, V, B> 
    where T: Tracer,
          V: VMTracer,
          B: StateBackend,
{
    fn push_snapshot(&mut self, interpreter: Box<dyn InterpreterExt + Send>) {
        self.snapshots.states.push(interpreter);
    }

    fn step_back(&mut self) -> Box<dyn InterpreterExt + Send> {
         if self.snapshots.states.len() <= 1 {
            self.snapshots.states.pop().unwrap()
        } else {
            // pop latest step
            self.snapshots.states.pop();
            // state = one step back
            self.snapshots.states.pop().unwrap()
        }
    }

    fn snapshots_len(&self) -> usize {
        self.snapshots.states.len()
    }

    fn externalities(&mut self) -> &mut dyn Ext {
        &mut self.externalities
    }
}

