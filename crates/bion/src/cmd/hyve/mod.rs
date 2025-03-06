use clap::{Parser, Subcommand};
use hyve_cli_runner::CliContext;
use list_operators::ListOperatorsCommand;
use onboard_operator::OnboardOperatorCommand;
use pause_operator::PauseOperatorCommand;
use register_operator::RegisterOperatorCommand;
use unpause_operator::UnpauseOperatorCommand;
use unregister_operator::UnregisterOperatorCommand;

mod list_operators;
mod onboard_operator;
mod pause_operator;
mod register_operator;
mod unpause_operator;
mod unregister_operator;

#[derive(Debug, Parser)]
#[clap(about = "Commands related to the HyveDA middleware.")]
pub struct HyveCommand {
    #[arg(value_name = "ALIAS", help = "The saved operator alias.")]
    alias: String,

    #[command(subcommand)]
    pub command: HyveSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum HyveSubcommands {
    #[command(name = "list-operators")]
    ListOperators(ListOperatorsCommand),

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

impl HyveCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        match self.command {
            HyveSubcommands::ListOperators(list_operators) => {
                list_operators.with_alias(self.alias).execute(ctx).await
            }
            HyveSubcommands::OnboardOperator(onboard_operator) => {
                onboard_operator.with_alias(self.alias).execute(ctx).await
            }
            HyveSubcommands::PauseOperator(pause_operator) => {
                pause_operator.with_alias(self.alias).execute(ctx).await
            }
            HyveSubcommands::RegisterOperator(register_operator) => {
                register_operator.with_alias(self.alias).execute(ctx).await
            }
            HyveSubcommands::UnpauseOperator(unpause_operator) => {
                unpause_operator.with_alias(self.alias).execute(ctx).await
            }
            HyveSubcommands::UnregisterOperator(unregister_operator) => {
                unregister_operator
                    .with_alias(self.alias)
                    .execute(ctx)
                    .await
            }
        }
    }
}
