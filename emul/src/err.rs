//! Error descriptions and implementations for Emulator 
use {std, vm, patricia_trie_ethereum as ethtrie};
use std::fmt;
use std::error;
use ethcore::error::ExecutionError;
use rayon::ThreadPoolBuildError;

/// Generic Error
/// Something happened in which Debugging cannot continue, but it cannot be attributed to
/// any one part of Emulator crate. This Error type should be used sparingly. Hopefully, never.
#[derive(Debug)]
pub struct GenericError;

/// An internal error occurred that is specific to EDB code
#[derive(Debug)]
pub struct InternalError(String);

#[derive(Debug)]
pub struct DebugError(String);

impl fmt::Display for DebugError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for DebugError {
    fn description(&self) -> &str {
        &(String::from("An error occured while debugging the program, causing the process to exit") + &self.0)
    }
}

impl InternalError {
    pub fn new(err: &str) -> Self {
        InternalError(err.to_owned())
    }
}

/// EVM Error. An error originated as vm::Error in Parity.
#[derive(Debug)]
pub struct EVMError(vm::Error);

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for InternalError {
    fn description(&self) -> &str {
        &(String::from("An Internal Error has occurred specific to EDB internal code. ") + &self.0)
    }
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An Error Occured OwO")
    }
}

impl error::Error for GenericError {
    fn description(&self) -> &str {
        "Something occurred and debugging cannot continue, but the error cannot be attributed \
            to any one part of the Emulator crate. This Error type should not be used often; \
            it is preferable to create a new error type if the Error is not covered in the \
            Emulators 'Error' Enum."
    }
}


impl fmt::Display for EVMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl error::Error for EVMError {
    fn description(&self) -> &str {
        "An error originating as vm::Error in Parity (vm crate, in ethcore)."
    }
}

#[derive(Debug)]
pub enum Error {
    // An error originating from Parity structures, (see: vm::Error)
    EVM(EVMError),
    Execution(ExecutionError),
    Internal(InternalError),
    Generic(GenericError),
    Thread(ThreadPoolBuildError),
    Debug(DebugError),
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::EVM(ref err) => write!(f, "EVM Error: {}", err),
            Error::Execution(ref err) => write!(f, "Execution Error: {}", err),
            Error::Internal(ref err) => write!(f, "Internal Error: {}", err),
            Error::Generic(ref err) => write!(f, "An Error Occurred OwO: {}", err),
            Error::Thread(ref err) => write!(f, "Error Building Threads: {}", err),
            Error::Debug(ref err) => write!(f, "Error Debuggin: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::EVM(ref err) => err.description(),
            Error::Execution(ref err) => err.description(),
            Error::Internal(ref err) => err.description(),
            Error::Generic(ref err) => err.description(),
            Error::Thread(ref err) => err.description(),
            Error::Debug(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Execution(ref err) => Some(err),
            Error::Internal(ref err) => Some(err),
            Error::Generic(ref err) => Some(err),
            Error::Thread(ref err) => Some(err),
            Error::Debug(ref err) => Some(err),
            Error::EVM(ref err) => Some(err),
        }
    }
}

impl From<DebugError> for Error {
    fn from(err: DebugError) -> Self {
        Error::Debug(err)
    }
}

impl From<ThreadPoolBuildError> for Error {
    fn from(err: ThreadPoolBuildError) -> Self {
        Error::Thread(err)
    }
}

impl From<Box<ethtrie::TrieError>> for Error {
    fn from(err: Box<ethtrie::TrieError>) -> Self {
        Error::EVM(EVMError(vm::Error::from(err)))
    }
}

impl From<ethtrie::TrieError> for Error {
    fn from(err: ethtrie::TrieError) ->  Self {
        Error::EVM(EVMError(vm::Error::from(err)))
    }
}

impl From<vm::Error> for Error {
    fn from(err: vm::Error) -> Self {
        Error::EVM(EVMError(err))
    }
}

impl From<Box<vm::Error>> for Error {
    fn from(err: Box<vm::Error>) -> Self {
        Error::EVM(EVMError(vm::Error::Internal(err.to_string())))
    }
}

impl From<InternalError> for Error {
    fn from(err: InternalError) -> Self {
        Error::Internal(err)
    }
}

impl From<ExecutionError> for Error {
    fn from(err: ExecutionError) -> Self {
        Error::Execution(err)
    }
}

impl From<Box<ExecutionError>> for Error {
    fn from(err: Box<ExecutionError>) -> Self {
        Error::Execution(ExecutionError::Internal(err.to_string()))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

