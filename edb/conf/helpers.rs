use log::*;

// panics if it fails because of anything other than the directory already exists
pub fn create_dir(path: std::path::PathBuf) {
    match std::fs::create_dir(path) {
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::AlreadyExists => (),
                _ => {
                    error!("{}", e);
                    std::process::exit(0x0100);
                }
            }
        },
        Ok(_) => ()
    }
}
