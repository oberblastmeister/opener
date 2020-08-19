use colored::Colorize;

pub fn print_error(msg: impl Into<String>) {
    eprintln!("{}: {}", "Error".bold().red(), msg.into())
}
