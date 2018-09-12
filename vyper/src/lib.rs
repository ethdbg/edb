mod err;
mod types;
mod binds;
use std::{
    path::PathBuf,
};
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn muck() {
        pretty_env_logger::init();
        let parsed = binds::parse(PathBuf::from("/home/insi/Projects/EDB/edb/tests/contracts/vyper/voting/voting.vy")).unwrap();
    }
}
