use clap::Subcommand;
use register::RegisterCommand;
use register_status::RegisterStatusCommand;

mod register;
mod register_status;

#[derive(Debug, Subcommand)]
pub enum OperatorCommands {
    #[command(name = "register")]
    Register(RegisterCommand),

    #[command(name = "register-status")]
    RegisterStatus(RegisterStatusCommand),
}
