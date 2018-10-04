use failure::{Fail, Error};

#[derive(Fail, Debug)]
pub enum CacheError {
    #[fail(display = "Node Error")]
    Node(String)
}

impl From<web3::error::Error> for CacheError {
    fn from(err: web3::error::Error) -> CacheError {
        CacheError::Node(format!("{}", err))
    }
}
