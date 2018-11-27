use std::str::FromStr;
use failure::Error;

use super::err::ShellError;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Help, // help message
    Clear, // Clear the screen
    Set, // set the params
    Run, // run a function
    Reset, // reset to first breakpoint
    Chain, // chain another tx
    Finish, // Finish current program (Next transaction keeps state from last transaction)
    Step, // step to next line
    Break, // toggle breakpoint
    Next, // go to next breakpoint
    Execute, // Execute to end (does not keep state from last transaction)
    Print, // Print variables n' stuff
    // Stack, // Show a representation the stack
    // Memory, // Show a representation of the memory
    // Storage, // show a representation of the storage
    Opcode, // show the current opcode
    Quit, // quit the debugger
    None, // no command
}

impl From<&Command> for String {
    fn from(command: &Command) -> String {
        match *command {
            Command::Help    => String::from("help"),
            Command::Clear   => String::from("clear"),
            Command::Set     => String::from("set"),
            Command::Run     => String::from("run"),
            Command::Reset   => String::from("reset"),
            Command::Chain   => String::from("chain"),
            Command::Finish  => String::from("finish"),
            Command::Step    => String::from("step"),
            Command::Break   => String::from("break"),
            Command::Next    => String::from("next"),
            Command::Execute => String::from("execute"),
            Command::Print   => String::from("print"),
            // Command::Stack   => String::from("stack"),
            // Command::Memory  => String::from("memory"),
            // Command::Storage => String::from("storage"),
            Command::Opcode  => String::from("opcode"),
            Command::Quit    => String::from("quit"),
            Command::None    => String::from("none"),
        }
    }
}

impl From<Command> for String {
    fn from(command: Command) -> String {
        match command {
            Command::Help    => String::from("help"),
            Command::Clear   => String::from("clear"),
            Command::Set     => String::from("set"),
            Command::Run     => String::from("run"),
            Command::Reset   => String::from("reset"),
            Command::Chain   => String::from("chain"),
            Command::Finish  => String::from("finish"),
            Command::Step    => String::from("step"),
            Command::Break   => String::from("break"),
            Command::Next    => String::from("next"),
            Command::Execute => String::from("execute"),
            Command::Print   => String::from("print"),
            // Command::Stack   => String::from("stack"),
            // Command::Memory  => String::from("memory"),
            // Command::Storage => String::from("storage"),
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
            "clear"           => Ok(Command::Clear),
            "set"             => Ok(Command::Set),
            "run"             => Ok(Command::Run),
            "reset"           => Ok(Command::Reset),
            "chain"           => Ok(Command::Chain),
            "finish"          => Ok(Command::Finish),
            "step"            => Ok(Command::Step),
            "break"           => Ok(Command::Break),
            "next"            => Ok(Command::Next),
            "execute"|"exec"  => Ok(Command::Execute),
            "print"           => Ok(Command::Print),
            // "stack"           => Ok(Command::Stack),
            // "memory"|"mem"    => Ok(Command::Memory),
            // "storage"|"storg" => Ok(Command::Storage),
            "opcode"|"op"     => Ok(Command::Opcode),
            "quit"|"exit"     => Ok(Command::Quit),
            _ => Err(ShellError::CommandNotFound(s.to_string()).into())
        }
    }
}
