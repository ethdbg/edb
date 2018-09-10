#![feature(specialization)]
mod vyper;
use std::path::PathBuf;

fn main() {
    let parsed = vyper::binds::parse(PathBuf::from("/home/insi/Projects/EDB/edb/tests/contracts/vyper/voting/voting.vy"));
    println!("PARSED: {}", parsed);
}



