use clap::Subcommand;
use create::CreateCommand;
use delete::DeleteCommand;
use export::ExportCommand;
use list::ListCommand;

pub mod create;
mod delete;
mod export;
mod list;

#[derive(Debug, Subcommand)]
#[clap(about = "Manage BLS keys for participating in the protocol.")]
pub enum BLSCommands {
    #[command(name = "create")]
    Create(CreateCommand),

    #[command(name = "delete")]
    Delete(DeleteCommand),

    #[command(name = "export")]
    Export(ExportCommand),

    #[command(name = "list")]
    List(ListCommand),
}
