//! A pretty simplistic implementation of a shell to use for EDB
mod commands;

use failure::Error;
use std::io::{stdin, stdout, Write};

use self::commands::*;
use self::ops::*;

fn shell() -> Result<(), Error> {

    'shell: loop {
        print!("> ");
        stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input)?;

        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap();
        let args = parts;
    }
}

fn commands(command: &str) {
    match command.parse() {
        Command::Help =>
    }
}

