
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Forward,
    Backward
}

impl Default for Direction {
    fn default() {
        Direction::Forward
    }
}

pub fn help() {
    unimplemented!();
}


pub fn step(dir: Option<Direction>, num: Option<usize>) {
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

/// Function to control default configuration of EDB. IE: How many lines to print at once, etc
pub fn config() {
    unimplemented!();
}

