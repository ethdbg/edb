use vm::{ActionParams, Ext};
// use evm::CostType;
use emulator::VMEmulator;
use ethcore::factory::VmFactory;
use extensions::evm_ext::factory_ext::FactoryExt as EvmFactoryExt;

pub trait FactoryExt {
    fn create_debug(&self, params: ActionParams, ext: &Ext) -> Box<VMEmulator + Send + Sync>;
}

impl FactoryExt for VmFactory {
    fn create_debug(&self, params: ActionParams, ext: &Ext) -> Box<VMEmulator + Send + Sync> {
        self.evm.create_debug(params, ext)
    }
}



