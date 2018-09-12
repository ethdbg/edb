use failure::Fail;

#[derive(Fail, Debug)]
pub enum VyperError {
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] std::io::Error),
    #[fail(display = "A Python Exception has occurred {}", _0)]
    Python(String),
    #[fail(display = "An error has occurred converting between Python and Rust types")]
    PyConversion(String),
}
impl From<pyo3::PyDowncastError> for VyperError {
    fn from(err: pyo3::PyDowncastError) -> VyperError { 
        VyperError::PyConversion(format!("{:?}", err))
    }
}

impl From<std::io::Error> for VyperError {
    fn from(err: std::io::Error) -> VyperError {
        VyperError::Io(err)    
    }
}

impl From<pyo3::PyErr> for VyperError {
    fn from(err: pyo3::PyErr) -> VyperError {
        VyperError::Python(format!("{:?}", err))
    }
}

pub type VyError<T> = Result<T, VyperError>;
