//! A pretty simplistic implementation of a shell to use for EDB
// providers job to compile the file

mod commands;
mod types;
mod ops;
mod err;
#[macro_use] mod helpers;

use failure::Error;
use log::*;
use ethereum_types::Address;
use termion::{
    input::TermRead,
    event::Key,
    raw::IntoRawMode,
};

use std::io::{stdin, stdout, Write};

use edb_core::{Debugger, Language, Transport, CompiledFiles};

use self::commands::Command;
use self::ops::*;
use self::err::*;
use super::lib::File;

pub struct Shell<T> where T: Transport {
    shell_history: Vec<String>,
    dbg: Option<Debugger<T>>,
    files: CompiledFiles, // TODO combine files with File struct
    client: web3::Web3<T>,
    addr: Address,
    root_file: File,
    current: Option<Vec<String>>
}
macro_rules! check {
    ($dbg:expr, $cmd: stmt) =>  ({
        if $dbg.is_none() {
            return Err(ShellError::Custom("Must run before using this command".to_string()).into());
        } else {
            $cmd
        }
    });
    ($dbg:expr) => ({
        if $dbg.is_none() {
            return Err(ShellError::Custom("Must run before using this command".to_string()).into());
        }
    })
}
// a simple shell
// Does nothing on no user input, will only crash with really fatal errors
// otherwise errors which are fixable are printed
impl<T> Shell<T> where T: Transport {

    pub fn new<L>(lang: L, client: web3::Web3<T>, addr: Address, file: File) -> Result<Self, Error> where L: Language {
        debug!("File: {:?}", file);
        Ok(Self {
            shell_history: Vec::new(),
            dbg: None,
            files: file.compile(lang, &addr)?,
            client, 
            addr, 
            root_file: file,
            current: None
        })
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

                match self.commands(command, parts) {
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

    // TODO: unecessary cloning
    fn commands<'a>(&mut self, command: Command, mut args: impl Iterator<Item = &'a str> + Clone) -> Result<(), Error> {
        
        match command {
            Command::Help    => help(args.next())?,
            Command::Clear   => clear()?,
            Command::Set     => {
                let a_c = args.clone();
                self.current = Some(a_c.map(|s| s.to_string()).collect::<Vec<String>>());
                let dbg = set(args, &self.root_file, self.files.clone(), self.addr.clone(), self.client.clone())?;
                self.dbg.replace(dbg);
            },
            Command::Run => {
                check!(self.dbg);
                let dbg = self.dbg.as_mut().unwrap();
                dbg.run()?;
            }
            Command::Reset   => {
                if self.current.is_none() {
                    return Err(ShellError::Custom("Must run before you can reset".to_string()).into());
                }
                let current = self.current.as_mut().unwrap();
                let dbg = set(current.iter().map(|s| s.as_str()), &self.root_file, self.files.clone(), self.addr.clone(), self.client.clone())?;
                self.dbg.replace(dbg);
            },
            Command::Chain   => { 
                if self.dbg.is_none() {
                    return Err(ShellError::Custom("Must run before you can reset".to_string()).into());
                }
                self.current = Some(args.clone().map(|s| s.to_string()).collect::<Vec<String>>());
                chain(&mut self.dbg.as_mut().unwrap(), self.files.clone(), args, &self.client, self.addr.clone())?;
            },
            Command::Finish  => finish(),
            Command::Step    => check!(self.dbg, step(&mut self.dbg.as_mut().unwrap(), args.next(), args.next())?),
            Command::Break   => br(&mut self.dbg.as_mut().unwrap(), args.next())?,
            Command::Next    => check!(self.dbg, next(&mut self.dbg.as_mut().unwrap())?),
            Command::Execute => execute(),
            Command::Print   => check!(self.dbg, print(&mut self.dbg.as_mut().unwrap(), args.next(), args.next())?),
            // Command::Stack   => stack(),
            // Command::Memory  => memory(),
            // Command::Storage => storage(),
            Command::Opcode  => opcode(),
            Command::Quit    => quit(),
            Command::None    => (),
        };
        Ok(())
    }
}


