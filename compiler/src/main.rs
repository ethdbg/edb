#![feature(specialization)]
mod vyper;
use std::path::PathBuf;
use self::vyper::err::VyError;

fn main() -> VyError<()> {
    pretty_env_logger::init();
    let parsed = vyper::binds::parse(PathBuf::from("/home/insi/Projects/EDB/edb/tests/contracts/vyper/voting/voting.vy"))?;
    Ok(())
}



