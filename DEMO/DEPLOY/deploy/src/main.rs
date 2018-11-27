extern crate rustc_hex;
extern crate web3;

use web3::futures::Future;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256};
use rustc_hex::FromHex;
use std::fs::File;
use std::io::Read;

fn main() {

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("no arguments");
    }
    let contract_name = args[1].clone();
    let addr = args[2].clone();
    let (mut raw_bytecode, mut raw_abi) = (String::new(), String::new());
    let mut b = File::open(format!("{}.bytecode", contract_name)).expect("Bytecode file missing");
    let mut a = File::open(format!("{}.abi", contract_name)).expect("ABI file missing");
    b.read_to_string(&mut raw_bytecode).expect("could not read bytecode file");
    a.read_to_string(&mut raw_abi).expect("Could not read abi file");

    let (_eloop, http) = web3::transports::Http::new("http://localhost:8545").expect("RPC not started");
    let web3 = web3::Web3::new(http);

    let my_account: Address = addr.parse().expect("Invalid addr");
    // Get the contract bytecode for instance from Solidity compiler
    let bytecode: Vec<u8> = raw_bytecode.from_hex().expect("Invalid bytecode");
    // Deploying a contract
    let contract = Contract::deploy(web3.eth(), raw_abi.as_bytes())
        .expect("Could not deploy step 1")
        .confirmations(0)
        .options(Options::with(|opt| {
            opt.value = Some(0.into())
        }))
        .execute(
            bytecode,
            (),
            my_account,
        )
        .expect("Correct parameters are passed to the constructor.")
        .wait()
        .expect("could not wait");
    println!("The contract is deployed at : {:x}", contract.address());
}

