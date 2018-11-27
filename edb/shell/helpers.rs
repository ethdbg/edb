use std::rc::Rc;

use sputnikvm::{HeaderParams, ValidTransaction, TransactionAction};
use edb_core::Transport;
use ethereum_types::{Address, U256};
use web3::types::{BlockNumber, BlockId};
use futures::future::Future;
use ethabi::{ParamType, Token};

use super::err::ShellError;

#[macro_export]
macro_rules! shell_error {
    ($msg: expr) => {
        use colored::*;
        eprint!("\n{}: {}", "Error".red().bold(), $msg);
    }
}

macro_rules! parse_err {
    ($name: expr) => ({
        ShellError::Custom(format!("invalid parameter for {}", $name))
    })
}

macro_rules! simple_err {
    ($name: expr) => ({
        ShellError::Custom(format!("Missing parameter for {}", $name))
    })
}

pub fn parse_args<'a>(func: &str, abi: &edb_core::Contract, mut args: impl Iterator<Item=&'a str>) -> Result<Vec<ethabi::Token>, ShellError> {
    let func = abi.function(func).map_err(|e| ShellError::Custom(format!("{}", e.description())))?;
    let mut param_tokens = Vec::new(); 
    for param in func.inputs.iter() {
        match &param.kind {
            ParamType::Address => {
                let arg = args.next();
                match arg {
                    Some(a) => param_tokens.push(Token::Address(a.parse().map_err(|e| parse_err!(param.name))?)),
                    None => { 
                        return Err(simple_err!(param.name));
                    }
                }
            },
            ParamType::Int(s) => {
                let arg = args.next();
                match arg {
                    Some(a) => param_tokens.push(Token::Int(a.parse().map_err(|e| parse_err!(param.name))?)),
                    None => {
                        return Err(simple_err!(param.name));
                    }
                }
            },
            ParamType::Uint(s) => {
                let arg = args.next();
                match arg {
                    Some(a) => param_tokens.push(Token::Uint(a.parse().map_err(|e| parse_err!(param.name))?)),
                    None => { 
                        return Err(simple_err!(param.name));
                    }
                }
            },
            ParamType::Bool => {
                let arg = args.next();
                match arg {
                    Some(a) => param_tokens.push(Token::Bool(a.parse().map_err(|e| parse_err!(param.name))?)),
                    None => { 
                        return Err(simple_err!(param.name));
                    }
                }
            },
            ParamType::String => {
                let arg = args.next();
                match arg {
                    Some(a) => param_tokens.push(Token::String(a.to_string())),
                    None => { 
                        return Err(simple_err!(param.name));
                    }
                }
            },
            ParamType::Bytes => unimplemented!(),
            ParamType::Array(_) => unimplemented!(),
            ParamType::FixedBytes(s) => unimplemented!(),
            ParamType::FixedArray(arr, s) => unimplemented!(),
            _ => panic!("Unknown Parameter Type")
        }
    }

    Ok(param_tokens)
}

// TODO should handle the hex case
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
        beneficiary: bigint::H160(block.author.0),
        timestamp: block.timestamp.as_u64(),
        number: bigint::U256(latest.0),
        difficulty: bigint::U256(block.difficulty.0),
        gas_limit: bigint::Gas::from(block.gas_limit.as_u64())
    }
}

// TODO: Proper 'transaction' interface that allows for more options (and setting options) like the
// index of acc to use, and tx params
// TODO: do proper error handling
// TODO: hide bigint/sputnikvm types. they clutter w/ ethereum types. Should not be needed when
// creating providers like shells/rpcs/daemons
// TODO: Edge cases not handled here (see Issue #29)
//
//
pub fn create_tx<T>(client: &web3::Web3<T>, addr: Address, abi: &edb_core::Contract, func: &str, params: &[ethabi::Token])
    -> Result<(HeaderParams, ValidTransaction), ShellError> where T: Transport 
{
    let func = abi.function(func)?.encode_input(params)?;
    let acc_zero = get_account(client, 0)?;
    let tx = ValidTransaction {
        caller: Some(bigint::H160(acc_zero.0)),
        gas_price: bigint::Gas::one(),
        gas_limit: bigint::Gas::from(1000000u64),
        action: TransactionAction::Call(bigint::H160(addr.0)),
        value: bigint::U256::zero(),
        input: Rc::new(func),
        nonce: bigint::U256::zero(),
    };
    Ok((get_headers(client), tx))
}

// TODO:  extend this to also get an account by an ID, not just index
pub fn get_account<T>(client: &web3::Web3<T>, idx: usize) -> Result<Address, ShellError> where T: Transport {
    let accounts = client.eth().accounts().wait().expect("Could not get accounts in `get_account` in shell helpers"); //TODO: handle error
    Ok(accounts[idx])
}


