use failure::Fail;


#[derive(Fail, Debug)]
pub enum ShellError {
    #[fail(display = "Command Not Found: `{}`",_0)]
    CommandNotFound(String),
    #[fail(display = "Could Not Decipher Direction: `{}`", _0)]
    DirectionNotFound(String),
    #[fail(display = "Need {} arguments for command `{}`", _0, _1)]
    ArgumentsRequired(usize, String),
    #[fail(display = "Could not get next input byte")]
    InputError,
    #[fail(display = "{}", _0)]
    Ethabi(String),
    #[fail(display = "{}", _0)]
    Custom(String),
}

impl From<ethabi::Error> for ShellError {
    fn from(err: ethabi::Error) -> ShellError {
        ShellError::Ethabi(format!("{}", err))
    }
}

