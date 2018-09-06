//! Emulates transaction execution and allows for real-time debugging
use log::{info, error, log};
use sputnikvm::{ValidTransaction, HeaderParams, SeqTransactionVM, errors::{RequireError, CommitError}, AccountCommitment, VM};
use futures::future::Future;
use sputnikvm_network_foundation::ByzantiumPatch;
use failure::Error;
use web3::{
    api::Web3,
    Transport,
    types::{BlockNumber, U256, Bytes, Address},
};
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use super::{
    err::EmulError,
};


/// An action or what should happen for the next step of execution
pub enum Action {
    /// step back one instruction
    StepBack,
    /// step forward one instruction
    StepForward,
    /// RunUntil a PC
    RunUntil(usize),
    /// finish instruction
    Finish,
    /// execute to the end
    Exec,
}


/// Emulation Object
pub struct Emulator<T: Transport> {
    vm: Rc<RefCell<SeqTransactionVM<ByzantiumPatch>>>,
    transaction: (ValidTransaction, HeaderParams),
    positions: Vec<usize>,
    client: web3::Web3<T>,
    ev_loop: tokio_core::reactor::Core,
}

/// a vm that emulates a transaction, allowing for mutations during execution
impl<T> Emulator<T> where T: Transport {
    /// Create a new Emulator
    pub fn new(transaction: ValidTransaction, header: HeaderParams, client: Web3<T>, ev_loop: tokio_core::reactor::Core) -> Self {
        Emulator {
            transaction: (transaction.clone(), header.clone()),
            vm: Rc::new(RefCell::new(sputnikvm::TransactionVM::new(transaction, header))),
            positions: Vec::new(),
            client,
            ev_loop,
        }
    }

    pub fn fire(&self, action: Action) -> Result<(), Error> {
        match action {
            Action::StepBack => self.step_back(),
            Action::StepForward => self.step_forward(),
            Action::RunUntil(pc) => self.run_until(pc),
            Action::Exec => self.run(),
            Action::Finish => self.run()
        }
    }

    fn output(&self) -> Vec<u8> {
        self.vm.borrow().out().into()
    }

    fn step_back(&self) -> Result<(), Error> {
        let mut last_pos = 0;
        if let Some(x) = self.positions.pop() {
            last_pos = x;
        }

        let mut pos_goal = 0;
        let (txinfo, header) = self.transaction;
        let mut new_vm = Rc::new(RefCell::new(sputnikvm::TransactionVM::new(txinfo, header)));
        while pos_goal < last_pos {
            step(new_vm.borrow_mut(), &self.client, &mut self.ev_loop)?;
            let state = new_vm.borrow().current_state().unwrap();
            pos_goal = state.position;
        }
        self.vm.swap(&new_vm);
        Ok(())
    }

    fn step_forward(&self) -> Result<(), Error> {
        step(self.vm.borrow_mut(), &self.client, &mut self.ev_loop)?;
        let state = self.vm.borrow().current_state().unwrap();
        self.positions.push(state.position);
        Ok(())
    }

    fn run_until(&self, pc: usize) -> Result<(), Error> {

        // If positions is 0, we haven't started the VM yet
        while *self.positions.get(self.positions.len()).unwrap_or(&0) < pc {
            step(self.vm.borrow_mut(), &self.client, &mut self.ev_loop)?;
            let state = self.vm.borrow().current_state().unwrap();
            self.positions.push(state.position);
        }
        Ok(())
    }

    /// runs vm to completion
    fn run(&self) -> Result<(), Error> {
        while !step(self.vm.borrow_mut(), &self.client, &mut self.ev_loop)? {}
        Ok(())
    }
}

/// steps the vm, querying node for any information that the VM needs
/// vm returns true when execution is finished
fn step<T>(vm: RefMut<SeqTransactionVM<ByzantiumPatch>>, client: &Web3<T>, ev_loop: &mut tokio_core::reactor::Core) -> Result<bool, EmulError>
where
    T: Transport
{
    match vm.step() {
        Ok(()) => {
            Ok(true)
        },
        Err(RequireError::Account(addr)) => {
            info!("Acquiring account {:#x} for VM", addr);
            let nonce = client.eth().transaction_count(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait()?;
            let balance: U256 = client.eth().balance(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait()?; // U256
            let code: Bytes = client.eth().code(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait()?; // Bytes

            let commit_res = vm.commit_account(AccountCommitment::Full {
                nonce: bigint::U256(nonce.0),
                address: addr,
                balance: bigint::U256(balance.0),
                code: Rc::new(code.0),
            })?;

            Ok(false)
        },
        Err(RequireError::AccountStorage(addr, index)) => {
            info!("Acquiring account storage at {:#x}, {:#x} for VM", addr, index);
            let value = client.eth().storage(ethereum_types::H160(addr.0), ethereum_types::U256(index.0), Some(BlockNumber::Latest)).wait()?;
            vm.commit_account(AccountCommitment::Storage {
                address: addr,
                index: index,
                // unsafe needs to be used here because bigint expects 4 u64's, while web3 function gives us an array of 32 bytes
                value: bigint::M256(bigint::U256(super::scary::non_scalar_typecast::h256_to_u256(value)))
            })?;
            Ok(false)
        },
        Err(RequireError::AccountCode(addr)) => {
            info!("Acquiring code at {:#x} for VM", addr);
            let code: Bytes = client.eth().code(ethereum_types::H160(addr.0), Some(BlockNumber::Latest)).wait()?;
            vm.commit_account(AccountCommitment::Code {
                address: addr,
                code: Rc::new(code.0)
            })?;
            Ok(false)
        },
        // the debugger is useless if execution cannot continue
        Err(err) => {
            error!("VM execution failed, unknown require! {:?}", err);
            panic!("VM Execution failed, unknown require");
        }
    }
}
