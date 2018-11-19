mod conf;
mod shell;

use log::*;
use failure::Error;

fn main() -> Result<(), Error> {
    let conf = conf::Configuration::new()?;
    self::shell::shell()?;
    Ok(())
}
