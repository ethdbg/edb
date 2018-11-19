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
    println!("{}", LOGO);
    println!("{}", WELCOME);
}

pub fn help() {
    unimplemented!();
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

/// Function to control default configuration of EDB. IE: How many lines to print at once, etc
pub fn config() {
    unimplemented!();
}

