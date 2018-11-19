

#[macro_export]
macro_rules! shell_error {
    ($msg: expr) => {
        use colored::*;
        eprintln!("{}: {}", "Error".red().bold(), $msg);
    }
}
