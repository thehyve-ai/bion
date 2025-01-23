use clap::Subcommand;
use get::GetCommand;
use list::ListCommand;
use opt_in::OptInCommand;
use opt_out::OptOutCommand;

mod get;
mod list;
mod opt_in;
mod opt_out;

#[derive(Debug, Subcommand)]
#[clap(about = "Commands for vault general use.")]
pub enum VaultCommands {
    #[command(name = "get")]
    Get(GetCommand),

    #[command(name = "list")]
    List(ListCommand),

    #[command(name = "opt-in")]
    OptIn(OptInCommand),

    #[command(name = "opt-out")]
    OptOut(OptOutCommand),
}
