mod cli;

use clap::Parser;
use cli::Cli;
use colored::Colorize;

fn main() {
    if let Err(err) = Cli::parse().run() {
        eprintln!("{}", format!("Error: {err:#}").bold().red());
        std::process::exit(1);
    }
}
