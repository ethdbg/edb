mod conf;
mod shell;

use failure::Error;

fn main() -> Result<(), Error> {
    let conf = conf::Configuration::new()?;
    self::shell::Shell::new().run()?;
    Ok(())
}
