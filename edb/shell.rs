//! A pretty simplistic implementation of a shell to use for EDB
// TODO: Should be implemented as a trait and not as a builder

mod commands;
mod types;
mod ops;
mod builder;
mod err;
#[macro_use] mod helpers;

use failure::Error;
use log::*;
use termion::{
    input::TermRead,
    event::Key,
    raw::IntoRawMode,
};

use std::{
    io::{stdin, stdout, Write},
    str::SplitWhitespace
};

use edb_core::{Debugger, Language, Transport, Solidity};

use self::commands::*;
use self::ops::*;
use self::err::*;
pub use self::builder::ShellBuilder;

pub struct Shell<T, L> where T: Transport, L: Language {
    shell_history: Vec<String>,
    dbg: Option<Debugger<T, L>>
}

// a simple shell
// Does nothing on no user input, will only crash with really fatal errors
// otherwise errors which are fixable are printed
impl<T, L> Shell<T, L> where T: Transport, L: Language {

    pub fn new() -> Self {
        Self {
            shell_history: Vec::new(),
            dbg: None,
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

                match commands(command, parts, self.dbg.as_mut()) {
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
            trace!("{:?}", c);
            match c {
                Key::Up => {
                    if entry >= self.shell_history.len() {
                        std::mem::replace(input, "".to_string());
                        write!(stdout, "{}{}{}{}", termion::cursor::Left(6u16), termion::clear::CurrentLine, "~> ", input)?;
                    } else {
                        entry += 1;
                        std::mem::replace(input, self.shell_history[self.shell_history.len() - entry].clone());
                        write!(stdout, "{}{}{}{}", termion::cursor::Left((input.len() + 6) as u16), termion::clear::CurrentLine, "~> ", input)?;
                    }
                },
                Key::Down => {
                    if entry == 0 {
                        std::mem::replace(input, "".to_string());
                        write!(stdout, "{}{}{}{}", termion::cursor::Left(6u16), termion::clear::CurrentLine, "~> ", input)?;
                    } else {
                        std::mem::replace(input, self.shell_history[self.shell_history.len() - entry].clone());
                        write!(stdout, "{}{}{}{}", termion::cursor::Left((input.len() + 6) as u16), termion::clear::CurrentLine, "~> ", input)?;
                        entry -= 1;
                    }
                },
                Key::Backspace => {
                    input.pop();
                    //let pos = stdout.cursor_pos()?;
                    write!(stdout,
                           "{}{}{}{}",
                           termion::clear::CurrentLine,
                           termion::cursor::Left((input.len() + 6) as u16),
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

fn commands<T, L>(command: Command, args: SplitWhitespace, dbg: Option<&mut Debugger<T, L>>) -> Result<(), Error>
where T: Transport,
      L: Language,
{
    match command {
        Command::Help    => help(),
        Command::Clear   => clear()?,
        Command::Run     => run(dbg, args)?,
        Command::Reset   => reset(),
        Command::Restart => restart(),
        Command::Finish  => finish(),
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

