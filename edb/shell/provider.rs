use edb_core::{Debugger, Solidity, Language, Transport};

pub struct Provider<T: Transport, L: Language> {
    file: Debugger<T, L>,
}

impl<T,L> Provider<T, L> where L: Language, T: Transport {
    pub fn new() -> Self {
        unimplemented!()
    }
}

