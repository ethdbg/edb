use failure::Error;
use web3::{Transport, types::{Address, BlockId, BlockNumber}};
use futures::future::Future;
use std::collections::HashSet;
use super::err::CacheError;

pub struct AddressCache {
    cache: HashSet<Address>,
    // Last scanned block
    last_block: u64,
}

impl AddressCache {
    /// Create a new Address cache
    pub fn new<T>(client: &web3::Web3<T>) -> Result<Self, CacheError> where T: Transport {
        let latest_block = client.eth().block_number().wait()?;
        let mut set = HashSet::new();
        Self::add(&mut set, latest_block.as_u64(), client)?;
        Ok(Self {
            cache: set,
            last_block: latest_block.as_u64(),
        })
    }

    /// Scan any new blocks that may have been created and add to set
    pub fn update<T>(&mut self, client: &web3::Web3<T>) -> Result<(), CacheError> where T: Transport {
        Self::add(&mut self.cache, self.last_block, client)?;
        Ok(())
    }

    fn add<T>(set: &mut HashSet<Address>, block: u64, client: &web3::Web3<T>) -> Result<(), CacheError> where T: Transport {
        for n in 0..=block {
            let block = client.eth().block(BlockId::Number(BlockNumber::Number(n))).wait()?;
            for hash in block.expect("Block should never be pending").transactions {
                let receipt = client.eth().transaction_receipt(hash).wait()?.expect("Receipt should never be pending");
                if let Some(addr) = receipt.contract_address {
                    set.insert(addr);
                }
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Address> {
        self.cache.iter()
    }

    pub fn as_vec(&self) -> Vec<Address> {
        self.cache.iter().map(|a| a.clone()).collect::<Vec<Address>>()
    }
}
