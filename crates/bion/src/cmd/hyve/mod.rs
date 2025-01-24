use bls::BLSCommands;
use clap::Subcommand;
use onboard_operator::OnboardOperatorCommand;
use pause_operator::PauseOperatorCommand;
use register_operator::RegisterOperatorCommand;
use unpause_operator::UnpauseOperatorCommand;
use unregister_operator::UnregisterOperatorCommand;

pub mod bls;
mod onboard_operator;
mod pause_operator;
mod register_operator;
mod unpause_operator;
mod unregister_operator;

#[derive(Debug, Subcommand)]
pub enum HyveCommands {
    #[command(name = "bls", subcommand)]
    BLS(BLSCommands),

    #[command(name = "onboard-operator")]
    OnboardOperator(OnboardOperatorCommand),

    #[command(name = "pause-operator")]
    PauseOperator(PauseOperatorCommand),

    #[command(name = "register-operator")]
    RegisterOperator(RegisterOperatorCommand),

    #[command(name = "unpause-operator")]
    UnpauseOperator(UnpauseOperatorCommand),

    #[command(name = "unregister-operator")]
    UnregisterOperator(UnregisterOperatorCommand),
}
