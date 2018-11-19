use std::str::FromStr;
use failure::Error;

use super::err::ShellError;

pub enum Command {
    Help,
    Step,
    Next,
    Execute,
    Print,
    Stack,
    Memory,
    Storage,
    Opcode,
    Quit,
    None,
}

impl From<&Command> for String {
    fn from(command: &Command) -> String {
        match *command {
            Command::Help    => String::from("help"),
            Command::Step    => String::from("step"),
            Command::Next    => String::from("next"),
            Command::Execute => String::from("execute"),
            Command::Print   => String::from("print"),
            Command::Stack   => String::from("stack"),
            Command::Memory  => String::from("memory"),
            Command::Storage => String::from("storage"),
            Command::Opcode  => String::from("opcode"),
            Command::Quit    => String::from("quit"),
            Command::None    => String::from("none"),
        }
    }
}



impl FromStr for Command {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let command = s.to_ascii_lowercase();

        match command.as_str() {
            "help"|"?"        => Ok(Command::Help),
            "step"            => Ok(Command::Step),
            "next"            => Ok(Command::Next),
            "execute"|"exec"  => Ok(Command::Execute),
            "print"           => Ok(Command::Print),
            "stack"           => Ok(Command::Stack),
            "memory"|"mem"    => Ok(Command::Memory),
            "storage"|"storg" => Ok(Command::Storage),
            "opcode"|"op"     => Ok(Command::Opcode),
            "quit"|"exit"     => Ok(Command::Quit),
            _ => Err(ShellError::CommandNotFound(s.to_string()).into())
        }
    }
}
