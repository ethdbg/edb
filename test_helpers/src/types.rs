//! Types/addresses/bytecode for contracts that different edb libraries test against
use std::{str::FromStr, path::PathBuf };
use log::*;

// use bigint::Address;
// ethereum_types::Address (H160)
pub enum Contract {
    SimpleStorage,
    Voting,
}

/// A caller stocked with ETH
pub const ADDR_CALLER: &'static str = "94143ba98cdd5a0f3a80a6514b74c25b5bdb9b59";

/// miner/beneficiary
pub const MINER: &'static str = "11f275d2ad4390c41b150fa0efb5fb966dbc714d";


// TODO: include import contracts
/// Simple storage bytecode
pub const SIMPLE_STORAGE_BYTECODE: &'static str         = include!("contracts/solidity/simple/SimpleStorage.bin");
/// Simple storage runtime bytecode
pub const SIMPLE_STORAGE_BYTECODE_RUNTIME: &'static str = include!("contracts/solidity/simple/SimpleStorage.bin-runtime");
/// Simple storage ABI
pub const SIMPLE_STORAGE_ABI: &'static [u8]             = include_bytes!("contracts/solidity/simple/simple.abi");

/// Voting bytecode
pub const VOTING_BYTECODE: &'static str         = include!("contracts/solidity/voting/Ballot.bin");
/// Voting runtime bytecode
pub const VOTING_BYTECODE_RUNTIME: &'static str = include!("contracts/solidity/voting/Ballot.bin-runtime");
/// Voting ABI
pub const VOTING_ABI: &'static [u8]             = include_bytes!("contracts/solidity/voting/Ballot.abi");

// Contract addressses
/// Simple storage contract address
pub const SIMPLE_STORAGE_ADDR: &'static str         = "0x884531eab1ba4a81e9445c2d7b64e29c2f14587c";
/// Simple storage import contract address
pub const SIMPLE_STORAGE_IMPORTS_ADDR: &'static str = "0x7205b1bb42edce6e0ced37d1fd0a9d684f5a860f";
/// Voting contract Address
pub const VOTING_ADDR: &'static str                 = "0x98a2559a814c300b274325c92df1682ae0d344e3";


/// returns the absolute path to contract
pub fn contract_path(contract: Contract) -> PathBuf {
    let path = match contract {
        Contract::SimpleStorage => "./../test_helpers/src/contracts/solidity/simple/simple.sol",
        Contract::Voting => "./../test_helpers/src/contracts/solidity/voting/voting.sol"
    };
    let relative = PathBuf::from(path);
    info!("Current Test Helper Dir: {:?}", std::env::current_dir().expect("Could not find current directory"));
    std::fs::canonicalize(relative).expect("Could not extract canonical path")
}


pub fn bigint_addr(val: &str) -> bigint::Address {
    bigint::Address::from_str(val).expect("Could not get address from str")
}

pub fn ethtype_addr(val: &str) -> ethereum_types::Address {
    let val = val.trim_left_matches("0x"); // from_str for ethereum_types doesn't like '0x' prefix
    ethereum_types::H160::from_str(val).expect("Could not get address from str")
}

pub fn abi(val: &[u8]) -> ethabi::Contract {
    ethabi::Contract::load(val).expect("Could not load contract abi from byte array")
}



