use failure::Fail;


#[derive(Fail, Debug)]
pub enum ShellError {
    #[fail(display = "Command Not Found: `{}`",_0)]
    CommandNotFound(String),
}

