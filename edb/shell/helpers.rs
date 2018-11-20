

#[macro_export]
macro_rules! shell_error {
    ($msg: expr) => {
        use colored::*;
        eprint!("\n{}: {}", "Error".red().bold(), $msg);
    }
}
