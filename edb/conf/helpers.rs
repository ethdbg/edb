use log::*;
use fern::colors::{Color, ColoredLevelConfig};

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

pub fn init_logger(level: log::LevelFilter) {
    let colors = ColoredLevelConfig::new()
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red)
        .debug(Color::Blue)
        .trace(Color::Magenta);

    let mut log_dir = dirs::data_local_dir()
        .expect("failed to find local data dir for logs");
    log_dir.push("edb");
    create_dir(log_dir.clone());
    log_dir.push("edb.logs");

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                    "{} [{}][{}] {} ::{};{}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    colors.color(record.level()),
                    message,
                    format_opt(record.file().map(|s| s.to_string())),
                    format_opt(record.line().map(|n| n.to_string()))
                ))
        })
        .chain(
            fern::Dispatch::new()
            .level(log::LevelFilter::Info)
            .level_for("edb_compiler", log::LevelFilter::Trace)
            .level_for("edb_emul", log::LevelFilter::Trace)
            .level_for("edb_core", log::LevelFilter::Trace)
            .level_for("edb", log::LevelFilter::Trace)
            .chain(fern::log_file(log_dir).expect("Failed to create edb.logs file"))
        )
        .chain(
            fern::Dispatch::new()
            .level(level)
            .chain(std::io::stdout())
        )
        .apply().expect("Could not init logging");
}

fn format_opt(file: Option<String>) -> String {
    match file {
        None => "".to_string(),
        Some(f) => f.to_string()
    }
}

