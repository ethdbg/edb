use failure::Fail;

#[derive(Fail, Debug, Clone)]
pub enum EmulError {
    #[fail(display = "An error occurred during the execution of the Emulator")]
    Execution,
    #[fail(display = "VM Error Occurred")]
    Vm(VmError),
    #[fail(display = "Web3 Error: {}", _0)]
    Web3(String)
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

#[derive(Debug, Clone)]
pub enum VmError {
    Commit(sputnikvm::errors::CommitError),
}
