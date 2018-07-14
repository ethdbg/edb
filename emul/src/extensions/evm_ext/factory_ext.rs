use vm;
use vm::{ActionParams, Ext};
//use evm::CostType;
use evm::factory::Factory;
use emulator::{Emulator, VMEmulator};
use ethereum_types::U256;

pub trait FactoryExt {
    fn create_debug(&self, params: ActionParams, ext: &Ext) -> vm::Result<Box<VMEmulator>>;
}

impl FactoryExt for Factory {

    /// Returns a debug interpreter
    /// Might be better to modify parity code and make another 'VMType' enum variant
    fn create_debug(&self, params: ActionParams, ext: &Ext) -> vm::Result<Box<VMEmulator>> {
        if can_fit_in_usize(&params.gas) {
            Ok(Box::new(Emulator::<usize>::new(params, self.evm_cache.clone(), ext)))
        } else  {
            Ok(Box::new(Emulator::<U256>::new(params, self.evm_cache.clone(), ext)))
        }
    }
}

fn can_fit_in_usize(gas: &U256) -> bool {
    gas == &U256::from(gas.low_u64() as usize)
}
