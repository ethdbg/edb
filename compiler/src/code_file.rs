use super::{Language, source_map::BytecodeSourceMap, SourceMap};
use web3::{contract::Contract, Transport};

// every CodeFile is associated with a language
pub struct CodeFile<L: Language, T: Transport> {
    /// Use Source Map// s
    source_maps: BytecodeSourceMap,
    /// Contracts contained in the file that can be deployed, or their abi queried
    contracts: Vec<Contract<T>>,
    /// Language Actions
    language: L
}

impl<L, T> CodeFile<L, T> where L: Language + SourceMap, T: Transport {
    pub fn new(lang: L, client: web3::Web3<T>) -> Self {

        Self {
            source_maps: BytecodeSourceMap::new(lang),
            contracts: Vec::new(),
            language: lang,
        }
    }
}
