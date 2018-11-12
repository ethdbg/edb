use failure::Fail;


#[derive(Fail, Debug)]
pub enum ConfigurationError {
    #[fail(display = "Error parsing CLI Arguments: {}", _0)]
    InputError(String)
}


impl From<FromHexError> for ConfigurationError {
    fn from(err: hex::FromHexError) -> ConfigurationError {
        ConfigurationError::InputError("Invalid Address {}", err)
    }
}
