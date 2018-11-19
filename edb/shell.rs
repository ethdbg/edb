//! A pretty simplistic implementation of a shell to use for EDB

mod commands;
mod types;
mod ops;
mod err;

use failure::Error;
use std::{
    io::{stdin, stdout, Write},
    str::SplitWhitespace
};

use self::commands::*;
use self::ops::*;

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
                    eprintln!("{}", e);
                    Command::None
                },
            };

            match commands(command, parts) {
                Ok(_)  => (),
                Err(e) => {
                    eprintln!("{}", e)
                }
            }
        } // do nothing on no input
    }
}

fn commands(command: Command, _args: SplitWhitespace) -> Result<(), Error> {
    match command {
        Command::Help    => help(),
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

