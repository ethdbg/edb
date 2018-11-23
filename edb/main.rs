mod conf;
mod shell;

use failure::Error;
// Get user input from Config
//  - (File Type)
//  - RPC


fn main() -> Result<(), Error> {
    let conf = conf::Configuration::new()?;
    //use provider not shell/rpc to start things
    //that way can plug in many different providers
    self::shell::Shell::new().run()?;
    Ok(())
}
