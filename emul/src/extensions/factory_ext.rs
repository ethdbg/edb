use vm::{ActionParams, Schedule};
use ethcore::factory::VmFactory;
use crate::extensions::evm_ext::factory_ext::FactoryExt as EvmFactoryExt;
use crate::emulator::VMEmulator;

crate trait FactoryExt {
    fn create_debug(&self, _: ActionParams, _: &Schedule, _: usize) -> Box<dyn VMEmulator + Send + Sync>;
}

impl FactoryExt for VmFactory {
    fn create_debug(&self, params: ActionParams, schedule: &Schedule, depth: usize) -> Box<dyn VMEmulator + Send + Sync> {
        self.evm.create_debug(params, schedule, depth)
    }
}



