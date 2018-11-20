//! A pretty simplistic implementation of a shell to use for EDB

mod commands;
mod types;
mod client;
mod provider;
mod err;
#[macro_use] mod helpers;

use failure::Error;
use log::*;
use termion::{
    input::TermRead,
    event::Key,
    raw::IntoRawMode,
    cursor::DetectCursorPos,
};

use std::{
    io::{stdin, stdout, Write, Read},
    str::SplitWhitespace
};

use self::commands::*;
use self::client::*;
use self::err::*;

pub struct Shell {
    shell_history: Vec<String>,
}

// a simple shell
// Does nothing on no user input, will only crash with really fatal errors
// otherwise errors which are fixable are printed
impl Shell {

    pub fn new() -> Self {
        Self {
            shell_history: Vec::new()
        }
    }

    pub fn run(mut self) -> Result<(), Error> {
        welcome();

        'shell: loop {
            print!("\n~> ");
            stdout().flush()?;

            let mut input = String::new();
            self.read_input(&mut input)?;
            self.shell_history.push(input.clone());
            let mut parts = input.trim().split_whitespace().to_owned();
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

    // parse events first, then if no events add key to buffer
    fn read_input(&self, input: &mut String) -> Result<(), Error> {
        let mut stdout = stdout().into_raw_mode()?;
        let mut entry: usize = 0;
        let mut cin = stdin().keys();
        'input: loop {
            let c = cin.next().ok_or(ShellError::InputError)??;
            debug!("{:?}", c);
            match c {
                Key::Up => {
                    if (self.shell_history.len() - entry) >= (self.shell_history.len()) {
                        std::mem::replace(input, "".to_string());
                    } else {
                        entry += 1;
                        std::mem::replace(input, self.shell_history[self.shell_history.len() - entry].clone());
                    }
                    print!("{}", input);
                    debug!("{}", input);
                },
                Key::Down => {
                    if entry == 0 {
                        std::mem::replace(input, "".to_string());
                    } else {
                        std::mem::replace(input, self.shell_history[self.shell_history.len() - entry].clone());
                        entry -= 1;
                    }
                    print!("{}", input);
                    debug!("{}", input);
                },
                Key::Backspace => {
                    input.pop();
                    //let pos = stdout.cursor_pos()?;
                    write!(stdout,
                           "{}{}{}{}",
                           termion::clear::CurrentLine,
                           termion::cursor::Left((input.len() + 4) as u16),
                           "~> ",
                           input
                           )?;
                },
                Key::Char('\n') => break,
                Key::Char(ch) => {
                    input.push(ch);
                    write!(stdout, "{}", ch)?;
                }
                _ => continue,
            };
            stdout.flush()?;
        }
        Ok(())
    }
}

fn commands(command: Command, mut args: SplitWhitespace) -> Result<(), Error> {
    match command {
        Command::Help    => help(),
        Command::Run     => {
            let arg0 = args.next().ok_or_else(|| ShellError::ArgumentsRequired(4, String::from(&command)))?;
            let arg1 = args.next().ok_or_else(|| ShellError::ArgumentsRequired(4, String::from(&command)))?;
            let arg2 = args.next().ok_or_else(|| ShellError::ArgumentsRequired(4, String::from(&command)))?;
            run(arg0, arg1, arg2, args);
        },
        Command::Step    => step(args.next(), args.next()),
        Command::Break   => br(args.next().ok_or_else(|| ShellError::ArgumentsRequired(1, String::from(command)))?),
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

