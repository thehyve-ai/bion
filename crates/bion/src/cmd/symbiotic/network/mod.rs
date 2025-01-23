use clap::Subcommand;
use opt_in::OptInCommand;
use opt_in_status::OptInStatusCommand;
use opt_out::OptOutCommand;

mod opt_in;
mod opt_in_status;
mod opt_out;

#[derive(Debug, Subcommand)]
pub enum NetworkCommands {
    #[command(name = "opt-in")]
    OptIn(OptInCommand),

    #[command(name = "opt-in-status")]
    OptInStatus(OptInStatusCommand),

    #[command(name = "opt-out")]
    OptOut(OptOutCommand),
}
