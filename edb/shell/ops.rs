//! All Operations the edb shell may execute are here

use failure::Error;
use termion::raw::IntoRawMode;
use ethereum_types::Address;
use log::*;

use std::{
    io::Write,
    str::{FromStr, SplitWhitespace},
};

use edb_core::{Debugger, CompiledFiles, Transport, contract::Find};

use crate::lib::File; // TODO: possibly move file out of configuration.
use super::commands::Command;
use super::types::*;
use super::err::ShellError;
use super::helpers::{self};

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Forward,
    Backward
}

pub enum Print {
    Result,
    Storage,
    Stack,
    Memory,
    Forward, // implies lines
    Backward, // implies lines
    Current
}

impl FromStr for Print {
    type Err = Error;
    fn from_str(s: &str) -> Result<Print, Error> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "res"|"result" => Ok(Print::Result),
            "storage"|"storg" => Ok(Print::Storage),
            "stack"|"s" => Ok(Print::Stack),
            "memory"|"mem" => Ok(Print::Memory),
            "forward"|"f" => Ok(Print::Forward),
            "backward"|"b" => Ok(Print::Backward),
            "current"|"curr" => Ok(Print::Current),
            _ => Err(ShellError::Custom("Unknown print value".to_string()).into())
        }
    }
}

impl FromStr for Direction {
    type Err = Error;
    fn from_str(s: &str) -> Result<Direction, Error> {
        match s {
            "forward"|"f" => Ok(Direction::Forward),
            "back"|"backward"|"b" => Ok(Direction::Backward),
            _ => Err(ShellError::DirectionNotFound(s.to_string()).into())
        }
    }
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::Forward
    }
}

/// the EDB Welcome Message
pub fn welcome() {
    let (longer, other) = {
        if LOGO.lines().count() > SHELL.lines().count() {
            (LOGO, SHELL)
        } else {
            (SHELL,LOGO)
        }
    };
    let iter_until = longer.lines().count() - other.lines().count();

    for (idx, line) in longer.lines().enumerate() {
        if idx < iter_until {
            println!("{}", line)
        } else {
            println!("{} {}", line, other.lines().nth(idx-iter_until).expect("Fatal error printing the welcome message"));
        }
    }
    print!("\n");
    println!("{}", WELCOME);
}

/// the help dialogue
pub fn help(subcommand: Option<&str>) -> Result<(), Error> {
    if let Some(s) = subcommand {
        match s.parse()? {
            Command::Help    => print!("\nDisplay the help message"),
            Command::Clear   => print!("\nClear the terminal"),
            Command::Run     => print!("\nRun"),
            Command::Reset   => print!("\nReset"),
            Command::Finish  => print!("\nFinish"),
            Command::Step    => print!("\nStep"),
            Command::Break   => print!("\nBreak"),
            Command::Next    => print!("\nNext"),
            Command::Execute => print!("\nExecute"),
            Command::Print   => print!("\nPrint vars"),
            // Command::Stack   => print!("\nStack"),
            // Command::Memory  => print!("\nMemory"),
            // Command::Storage => print!("\nStorage"),
            Command::Opcode  => print!("\nOpcode"),
            Command::Quit    => print!("\nQuit"),
            _=> (),
        }
    } else {
        println!("{}", HELP);
    }
    Ok(())
}

/// clears the terminal
pub fn clear() -> Result<(), Error> {
    let mut stdout = std::io::stdout().into_raw_mode()?;
    write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1,1))?;
    Ok(())
}

// TODO: get rid of the 'file' argument altogether. requires refactoring in core/compiler/down the
// chain
// need the function ABI to be able to match params
// pub fn run(contract: &str, func: &str, params: SplitWhitespace) {
pub fn set<'a, T>(mut params: impl Iterator<Item=&'a str> + Clone, 
            file: &File,
            files: CompiledFiles,
            addr: Address, 
            client: web3::Web3<T>) 
-> Result<Debugger<T>, Error> where T: Transport
{
    let contract = params.next().ok_or(ShellError::Custom("need to specify a contract".to_string()))?;
    let func = params.next().ok_or(ShellError::Custom("Need to specify function to run".to_string()))?;

    debug!("Files: {:?}", files);
    debug!("Running {} {}", contract, func);
    let contract = files.contracts().find(contract)?;

    let mut contract_args;
    let param_clone = params.clone();
    if param_clone.peekable().peek().is_some() {
        contract_args = helpers::parse_args(func, &contract, params)?;
    } else {
        contract_args = Vec::new();
    }

    let (block, tx) = helpers::create_tx(&client, addr, contract, func, contract_args.as_slice())?;
    Ok(Debugger::new(file.path(), files.clone(), client.clone(), tx, block, contract.name())?)
}

// chain tx to existing debugger
pub fn chain<'a, T>(dbg: &mut Debugger<T>,
                    files: CompiledFiles,
                    mut params: impl Iterator<Item=&'a str> + Clone,
                    client: &web3::Web3<T>,
                    addr: Address)
-> Result<(), Error> where T: Transport
{
    let contract = params.next().ok_or(ShellError::Custom("need to specify a contract".to_string()))?;
    let func = params.next().ok_or(ShellError::Custom("Need to specify a function to run".to_string()))?;
    debug!("Chaining {} {}", contract, func);
    let contract = files.contracts().find(contract)?;

    let mut contract_args;
    let param_clone = params.clone();
    if param_clone.peekable().peek().is_some() {
        contract_args = helpers::parse_args(func, &contract, params)?;
    } else {
        contract_args = Vec::new();
    }

    let (block, tx) = helpers::create_tx(client, addr, contract, func, contract_args.as_slice())?;
    Ok(dbg.chain(tx, Some(block)))

}

pub fn reset() {
    unimplemented!()
}

pub fn finish() {
    unimplemented!()
}

pub fn step<T: Transport>(dbg: &mut Debugger<T>, direction: Option<&str>, num: Option<&str>) -> Result<(), Error> {
    let num = num.unwrap_or("1").parse().map_err(|_| ShellError::Custom(format!("`{}` is not valid. Must be a positive integer from 0 to 2^32", num.unwrap())))?;
    let direction = direction.unwrap_or("forward");
    debug!("Stepping {} lines", num);
    match direction.parse()? {
        Direction::Forward => {
            for _ in 0..=num {
                dbg.step_forward()?;
            }
        },
        Direction::Backward => {
            return Err(ShellError::Custom(format!("{}", "Stepping backward not yet stable")).into());
        }
    }
    Ok(())
}

/// set breakpoints
pub fn br<T>(dbg: &mut Debugger<T>, line: Option<&str>) -> Result<(), Error> where T: Transport {
    if let Some(bp) = line {
        let bp = bp.parse()?;
        dbg.set_breakpoint(bp)?;
        Ok(())
    } else {
        return Err(ShellError::ArgumentsRequired(1, "break".to_string()).into());
    }
}

pub fn next<T>(dbg: &mut Debugger<T>) -> Result<(), Error> where T: Transport {
    dbg.next()?;
    Ok(())
}

pub fn execute() {
    unimplemented!();
}

pub fn print<T>(dbg: &mut Debugger<T>, item: Option<&str>, num: Option<&str>) -> Result<(), Error> where T: Transport {
    if item.is_none() {
        println!("\n{}", dbg.current_range()?);
    } else {
        let num = num.unwrap_or("1").parse().map_err(|_| ShellError::Custom(format!("`{}` is not valid. Must be a positive integer from 0 to 2^32", num.unwrap())))?;
        match item.expect("scope is conditional; qed").parse()? {
            Print::Current => {
                let (line, stri) = dbg.current_line()?;
                println!("\n{}: {}", line, stri);
            },
            Print::Forward => {
                let lines = dbg.next_lines(num)?;
                for (nu, line) in lines.iter() {
                    println!("\n{}: {}", nu, line);
                }
            },
            Print::Backward => {
                let lines = dbg.last_lines(num)?;
                for (nu, line) in lines.iter() {
                    println!("\n{}: {}", nu, line);
                }
            },
            Print::Stack => {
                let stack = dbg.stack()?;
                stack.iter().enumerate().for_each(|(i, x)| {
                    println!("\nitem {}: {:#x}, b10: {};", i, x, x);
                })
            },
            Print::Memory => {
                let mem = dbg.memory()?;
                mem.iter().enumerate().for_each(|(i, x)| {
                    println!("\nitem: {}: {:#x};", i, x);
                })
            },
            Print::Storage => {
                let storg = dbg.storage();
                if storg.is_none() {
                    println!("\nNo storage has been committed yet!");
                } else {
                    let storg = storg.expect("Scope is conditional");
                    let mut tmp: Vec<(&bigint::U256, &bigint::M256)> = storg.iter().collect();
                    tmp.sort_by(|a, b| a.cmp(b));
                    tmp.iter().for_each(|(k, v)| {
                        println!("\n{}: 0x{:#x}", k, v);
                    });
                }
            },
            Print::Result => {
                println!("\n");
                dbg.output()
                    .as_slice()
                    .iter()
                    .for_each(|x| {
                        print!("{:#x} ", x)
                    });
                println!("\n");
            }
        }
    }
    Ok(())
}
/*
pub fn stack() {
    unimplemented!();
}

pub fn memory() {
    unimplemented!();
}

pub fn storage() {
    unimplemented!();
}
*/
pub fn opcode() {
    unimplemented!();
}

pub fn quit() {
    println!("\nGoodbye!");
    std::process::exit(0);
}


/****
 *    /\/\/\/\/\/\/\/\/\//\/\/\/\/\/\/\
 *      TO BE IMPLEMENTED IF TIME LEFT (Mostly QoL things)
 *   \/\/\/\/\\/\/\\\/\\/\\\/\/\/\/\/\/\/\
 */
/// Function to control default configuration of EDB. IE: How many lines to print at once, etc
/// ie `config edb xxxx` <-- EDB/ui specific
/// or `config eth xxx` <--- ethereum specific (block num, gas limit, tx params, etc etc)
///         eth: account to use (acc 0)
pub fn config() {
    unimplemented!();
}

/// Import addresses from a file
/// OR crawl all addresses on testRPC (there should be a function to check if we are actually
/// dealing with a TestRPC) and list them for the user to select from
pub fn import() {
    unimplemented!();
}

/// useful filesystem functions
/// EX => `ls`, `cd`, `cat`, `grep`
pub fn os_api() {
    unimplemented!();
}

#[cfg(test)]
mod tests {


}

