use failure::Fail;


#[derive(Fail, Debug)]
pub enum ConfigurationError {
    #[fail(display = "Parsing CLI Arguments: {}", _0)]
    InputError(String),
}


impl From<hex::FromHexError> for ConfigurationError {
    fn from(err: hex::FromHexError) -> ConfigurationError {
        ConfigurationError::InputError("Invalid Address".to_string())
    }
}
