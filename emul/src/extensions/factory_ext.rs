use vm::{ActionParams, Ext};
use ethcore::factory::VmFactory;
use crate::extensions::evm_ext::factory_ext::FactoryExt as EvmFactoryExt;
use crate::emulator::VMEmulator;

pub trait FactoryExt {
    fn create_debug(&self, params: ActionParams, ext: &Ext) -> Box<VMEmulator + Send + Sync>;
}

impl FactoryExt for VmFactory {
    fn create_debug(&self, params: ActionParams, ext: &Ext) -> Box<VMEmulator + Send + Sync> {
        self.evm.create_debug(params, ext)
    }
}



