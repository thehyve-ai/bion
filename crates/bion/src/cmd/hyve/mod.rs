use clap::{Parser, Subcommand};
use hyve_cli_runner::CliContext;
use onboard_operator::OnboardOperatorCommand;
use register_operator::RegisterOperatorCommand;

mod onboard_operator;
mod register_operator;

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
    #[command(name = "onboard-operator")]
    OnboardOperator(OnboardOperatorCommand),

    #[command(name = "register-operator")]
    RegisterOperator(RegisterOperatorCommand),
}

impl HyveCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        match self.command {
            HyveSubcommands::OnboardOperator(onboard_operator) => {
                onboard_operator.with_alias(self.alias).execute(ctx).await
            }
            HyveSubcommands::RegisterOperator(register_operator) => {
                register_operator.with_alias(self.alias).execute(ctx).await
            }
        }
    }
}
