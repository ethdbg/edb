use failure::{Fail, Error};

#[derive(Fail, Debug)]
pub enum CacheError {
    #[fail(display = "Node Error")]
    Node(String),
}

impl From<web3::error::Error> for CacheError {
    fn from(err: web3::error::Error) -> CacheError {
        CacheError::Node(format!("{}", err))
    }
}

#[derive(Fail, Debug)]
pub enum EvmError {
    #[fail(display = "VM not initialized. Run first before doing anything")]
    NotInitialized,
    #[fail(display = "Stack Error {:?}", _0)]
    StackError(sputnikvm::errors::OnChainError)
}

impl From<sputnikvm::errors::OnChainError> for EvmError {
    fn from(err: sputnikvm::errors::OnChainError) -> EvmError {
        EvmError::StackError(err)
    }
}
