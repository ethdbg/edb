use std::str::FromStr;
use failure::Error;

pub enum Command {
    Help,
    Step,
    Exec,
    Print,
    Stack,
    Memory,
    Storage,
    Opcode
}

impl From<&Command> for String {
    fn from(command: &Command) -> String {
        match *command {
            Command::Help    => String::from("help"),
            Command::Step    => String::from("step"),
            Command::Exec    => String::from("exec"),
            Command::Print   => String::from("print"),
            Command::Stack   => String::from("stack"),
            Command::Memory  => String::from("memory"),
            Command::Storage => String::from("storage"),
            Command::Opcode  => String::from("opcode")
        }
    }
}



impl FromStr for Command {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let command = s.to_ascii_lowercase();

        Ok(match command.as_str() {
            "help" => Command::Help,
            "step" => Command::Step,
            "execute"|"exec" => Command::Exec,
            "print" => Command::Print,
            "stack" => Command::Stack,
            "memory"|"mem" => Command::Memory,
            "storage"|"storg" => Command::Storage,
            "opcode"|"op" => Command::Opcode
        })
    }
}
