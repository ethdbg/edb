#![feature(duration_as_u128)]
mod conf;
mod ui;

use log::*;
use failure::Error;

fn main() -> Result<(), Error> {
    let conf = conf::Configuration::new()?;
    ui::launch_tui()?;
    Ok(())
}
