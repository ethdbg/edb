
//! EVM factory
use std::sync::Arc;
use vm::{self, Vm};
use ethereum_types::U256;
use vm::ActionParams;
use evm::interpreter::{SharedCache, Interpreter};
use evm::{CostType};

/// Like Parities Factory, but returns full Interpreter type, and not just vm::Vm
/// Interpreter type also not in Box<>
pub struct Factory {
    evm_cache: Arc<SharedCache>,
}

impl Factory {

    pub fn create(&self, params: ActionParams, ext: &vm::Ext) -> vm::Result<Interpreter<U256>> {
        Ok(Interpreter::<U256>::new(params, self.evm_cache.clone(), ext)?)
    }

    pub fn new(cache_size: usize) -> Self {
        Factory {
            evm_cache: Arc::new(SharedCache::new(cache_size)),
        }
    }
}

