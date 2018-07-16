use std::fmt;
use std::error;
use vm;

#[derive(Debug)]
pub struct GenericError;

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
pub enum Error {
    // An error originating from Parity structures, (see: vm::Error)
    EVM(vm::Error),
    // Something happened in which Debugging cannot continue, but it cannot be attributed to
    // any one part of Emulator crate. This Error type should be used sparingly. Hopefully, never.
    Generic(String)
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::EVM(ref err) => write!(f, "EVM Error: {}", err.to_string()),
            Error::Generic(ref err) => write!(f, "An Error Occurred OwO: {}", err)
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::EVM(ref err) => write!(f, "EVM Error {}", err.to_string()),
            Error::Generic(ref err) => write!(f, "")
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            
        }
    }

}

impl From<vm::Error> for Error {
    fn from(err: vm::Error) -> Self {
        Error::EVM(err)
    }
}

impl From<Box<vm::Error>> for Error {
    fn from(err: Box<vm::Error>) -> Self {
        Error::EVM(err)
    }
}
