use failure::{Fail, Error};

#[derive(Fail, Debug)]
pub enum VyperError {
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] std::io::Error),
    #[fail(display = "A Python Exception has occurred")]
    Python(#[fail(cause)] pyo3::PyErr)
}

impl From<std::io::Error> for VyperError {
    fn from(err: std::io::Error) -> VyperError {
        VyperError::Io(err)    
    }
}

impl From<pyo3::PyErr> for VyperError {
    fn from(err: pyo3::PyErr) -> VyperError {
        VyperError::Python(err)
    }
}

type VyError<T> = Result<T, VyperError>;
