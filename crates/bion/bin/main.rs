mod cli;

use clap::Parser;
use cli::Cli;

fn main() {
    if let Err(err) = Cli::parse().run() {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
