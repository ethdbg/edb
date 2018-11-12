mod conf;

use log::*;
use failure::Error;

fn main() -> Result<(), Error> {
    let conf = conf::Configuration::new()?;
    Ok(())
}
