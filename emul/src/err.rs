use failure::Fail;


/// Top  level Emulator  errors
#[derive(Fail, Debug)]
pub enum EmulError {
    #[fail(display = "An error occurred during the execution of the Emulator")]
    Execution,
    #[fail(display = "VM Error Occurred, _0")]
    Vm(VmError),
    #[fail(display = "Web3 Error: {}", _0)]
    Web3(String),
    #[fail(display = "An error occurred storing or retrieving data for an ethereum account from local storage {}", _0)]
    State( #[fail(cause)] StateError),
}

/// Errors that occured while interacting with In-Memory or cached Ethereum State Storage
#[derive(Fail, Debug)]
pub enum StateError {
    #[fail(display = "IO Error")]
    Io(std::io::Error),
    #[fail(display = "Could not find account entry corresponding to {}", _0)]
    NotFound(bigint::H160),
}

impl From<std::io::Error> for EmulError {
    fn from(err: std::io::Error) -> EmulError {
        EmulError::State(StateError::Io(err))
    }
}

impl From<sputnikvm::errors::CommitError> for EmulError {
    fn from(err: sputnikvm::errors::CommitError) -> EmulError {
        EmulError::Vm(VmError::Commit(err))
    }
}

impl From<web3::Error> for EmulError {
    fn from(err: web3::Error) -> EmulError {
        EmulError::Web3(format!("{}", err))
    }
}

#[derive(Fail, Debug, Clone)]
pub enum VmError {
    #[fail(display = "Commit {:?}", _0)]
    Commit(sputnikvm::errors::CommitError),
}
