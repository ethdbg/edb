//! A pretty simplistic implementation of a shell to use for EDB

mod commands;
mod types;
mod client;
mod provider;
mod err;
#[macro_use]
mod helpers;

use failure::Error;
use log::*;

use std::{
    io::{stdin, stdout, Write},
    str::SplitWhitespace
};

use self::commands::*;
use self::client::*;
use self::helpers::*;

// a simple shell
// Does nothing on no user input, will only crash with really fatal errors
// otherwise errors which are fixable are printed
pub fn shell() -> Result<(), Error> {
    welcome();
    'shell: loop {
        print!("~> ");
        stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input)?;

        let mut parts = input.trim().split_whitespace();
        if let Some(command) = parts.next() {
            let command = match command.parse() {
                Ok(v) => v,
                Err(e) => {
                    shell_error!(e);
                    Command::None
                },
            };

            match commands(command, parts) {
                Ok(_)  => (),
                Err(e) => {
                    shell_error!(e);
                }
            }
        } // do nothing on no input
    }
}

fn commands(command: Command, _args: SplitWhitespace) -> Result<(), Error> {
    match command {
        Command::Help    => help(),
        Command::Run     => unimplemented!(),
        Command::Step    => step(None, None),
        Command::Next    => next(),
        Command::Execute => execute(),
        Command::Print   => print(None, None),
        Command::Stack   => stack(),
        Command::Memory  => memory(),
        Command::Storage => storage(),
        Command::Opcode  => opcode(),
        Command::Quit    => quit(),
        Command::None    => (),
    };

    Ok(())
}

