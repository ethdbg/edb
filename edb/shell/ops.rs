//! All Operations the edb shell may execute are here
use ethereum_types::H160;
use failure::Error;

use termion::raw::IntoRawMode;

use std::{
    io::Write,
    str::{FromStr, SplitWhitespace},
};
use edb_core::{Debugger, Solidity, Language, Transport};

use super::types::*;
use super::err::ShellError;

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Forward,
    Backward
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
            println!("{} {}", line, other.lines().nth(idx-iter_until).expect("Fatal error"));
        }
    }
    print!("\n");
    println!("{}", WELCOME);
}

pub fn help() {
    println!("{}", HELP);
}

pub fn clear() -> Result<(), Error> {
    let mut stdout = std::io::stdout().into_raw_mode()?;
    write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1,1))?;
    Ok(())
}

// need the function ABI to be able to match params
// pub fn run(address: &str, contract: &str, func: &str, params: SplitWhitespace) {
pub fn run<T: Transport, L: Language>(file: &mut Debugger<T, L>, params: SplitWhitespace) -> Result<(), Error> {
    unimplemented!();
}

pub fn reset() {
    unimplemented!()
}

pub fn restart() {
    unimplemented!()
}

pub fn finish() {
    unimplemented!()
}

pub fn step(dir: Option<&str>, num: Option<&str>) {
    unimplemented!();
}

/// set breakpoints
pub fn br(line: &str) {
    unimplemented!();
}

pub fn next() {
    unimplemented!();
}

pub fn execute() {
    unimplemented!();
}

pub fn print(dir: Option<Direction>, num: Option<usize>) {
    unimplemented!();
}

pub fn stack() {
    unimplemented!();
}

pub fn memory() {
    unimplemented!();
}

pub fn storage() {
    unimplemented!();
}

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
pub fn config() {
    unimplemented!();
}

/// Import addresses from a file
/// OR crawl all addresses on testRPC (there should be a function to check if we are actually
/// dealing with a TestRPC) and list them for the user to select from
pub fn import() {
    unimplemented!();
}


