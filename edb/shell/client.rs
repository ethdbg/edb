//! All Operations the edb shell may execute are here
use ethereum_types::H160;
use ethabi;
use super::types::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Forward,
    Backward
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::Forward
    }
}


// the EDB Welcome Message
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

pub fn run(address: H160, func: &str, params: Vec<ethabi::Param>) {
    unimplemented!()
}

pub fn step(dir: Option<Direction>, num: Option<usize>) {
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
    println!("Goodbye!");
    std::process::exit(0);
}


/****
 *    /\/\/\/\/\/\/\/\/\//\/\/\/\/\/\/\
 *      TO BE IMPLEMENTED IF TIME LEFT (Mostly QoL things)
 *   \/\/\/\/\\/\/\\\/\\/\\\/\/\/\/\/\/\/\
 */
/// Function to control default configuration of EDB. IE: How many lines to print at once, etc
/// ie `config edb xxxx` <-- EDB/ui specific 
/// or `config eth xxx` <--- ethereum specific (block num etc)
pub fn config() {
    unimplemented!();
}

/// Import addresses from a file
/// OR crawl all addresses on testRPC (there should be a function to check if we are actually
/// dealing with a TestRPC) and list them for the user to select from
pub fn import() {
    unimplemented!();
}


