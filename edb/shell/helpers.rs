use std::str::SplitWhitespace;

use sputnikvm::{HeaderParams, ValidTransaction, TransactionAction};
use edb_core::Transport;
use ethereum_types::Address;

#[macro_export]
macro_rules! shell_error {
    ($msg: expr) => {
        use colored::*;
        eprint!("\n{}: {}", "Error".red().bold(), $msg);
g   }
}


pub fn parse_args(args: SplitWhitespace) -> Vec<ethabi::Param> {
    unimplemented!();
}

// should handle the hex case
pub fn to_num() -> () {
    unimplemented!()
}


// TODO: do proper error handling
// TODO: hide bigint/sputnikvm types. they clutter w/ ethereum types. Should not be needed when
// creating providers like shells/rpcs/daemons
// TODO: Edge cases not handled here (see Issue #29)
//
// just use params from latest block for header
fn get_headers<T>(client: &web3::Web3<T>) -> HeaderParams where T: Transport {
    let block = client.eth().block(BlockId::Number(BlockNumber::Latest)).wait().expect("Could not get latest block").expect("Failed getting latest block");
    let latest = client.eth().block_number().wait().expect("Could not query latest block");
    
    HeaderParams {
        beneficiary: bigint::H160::(block.author.0),
        timestamp: block.timestamp.as_u64(),
        number: bigint::U256(latest.0),
        difficulty: bigint::U256(block.difficulty.0),
        gas_limit: bigint::Gas::from(block.gas_limit.as_u64())
    }
}

// TODO: do proper error handling
// TODO: hide bigint/sputnikvm types. they clutter w/ ethereum types. Should not be needed when
// creating providers like shells/rpcs/daemons
// TODO: Edge cases not handled here (see Issue #29)
//
//
pub fn create_tx<T>(client: &web3::Web3<T>, addr: Address, abi: ethabi::Contract) -> (HeaderParams, ValidTransaction) where T: Transport {
    

}
