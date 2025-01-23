use clap::Subcommand;
use get::GetCommand;
use list::ListCommand;
use opt_in::OptInCommand;
use opt_in_status::OptInStatusCommand;
use opt_out::OptOutCommand;

mod get;
mod list;
mod opt_in;
mod opt_in_status;
mod opt_out;

#[derive(Debug, Subcommand)]
pub enum VaultCommands {
    #[command(name = "get")]
    Get(GetCommand),

    #[command(name = "list")]
    List(ListCommand),

    #[command(name = "opt-in")]
    OptIn(OptInCommand),

    #[command(name = "opt-in-status")]
    OptInStatus(OptInStatusCommand),

    #[command(name = "opt-out")]
    OptOut(OptOutCommand),
}
