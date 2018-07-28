use vm::{ActionParams, Schedule};
use evm::factory::Factory;
use ethereum_types::U256;
use crate::emulator::{Emulator, VMEmulator};

crate trait FactoryExt {
    fn create_debug(&self, params: ActionParams, schedule: &Schedule, depth: usize) -> Box<dyn VMEmulator + Send + Sync>;
}

impl FactoryExt for Factory {

    /// Returns a debug interpreter
    /// Might be better to modify parity code and make another 'VMType' enum variant
    fn create_debug(&self, params: ActionParams, schedule: &Schedule, depth: usize) -> Box<dyn VMEmulator + Send + Sync> {
        if can_fit_in_usize(&params.gas) {
            Box::new(Emulator::<usize>::new(params, self.evm_cache.clone(), schedule, depth))
        } else  {
            Box::new(Emulator::<U256>::new(params, self.evm_cache.clone(), schedule, depth))
        }
    }
}

fn can_fit_in_usize(gas: &U256) -> bool {
    gas == &U256::from(gas.low_u64() as usize)
}
