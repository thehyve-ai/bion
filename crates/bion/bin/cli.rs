use bion::cmd::{
    network::NetworkCommands,
    operator::{bls::BLSCommands, OperatorCommands},
    vault::VaultCommands,
};
use clap::{
    builder::{styling::AnsiColor, Styles},
    ArgAction, Parser, Subcommand, ValueEnum,
};
use hyve_cli_runner::CliRunner;
use hyve_version::SHORT_VERSION;

#[derive(ValueEnum, Clone, Debug)]
pub enum Networks {
    #[value(alias("sepolia"))]
    Sepolia,
}

impl Networks {
    pub fn as_str(&self) -> &str {
        match self {
            Networks::Sepolia => "sepolia",
        }
    }
}

/// The verbosity level.
pub type Verbosity = u8;

#[derive(Debug, Parser)]
#[command(
    name = "hyve",
    about = "High-throughput data availability committee node for the HyveDA network.",
    author = "Hyve <support@thehyve.xyz>",
    version = SHORT_VERSION.as_str(),
    term_width = 80,
    styles = get_color_style()
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true, help = "Enable debug logging")]
    pub debug: bool,

    /// Verbosity level of the log messages.
    ///
    /// Pass multiple times to increase the verbosity (e.g. -v, -vv, -vvv).
    ///
    /// Depending on the context the verbosity levels have different meanings.
    ///
    /// For example, the verbosity levels of the EVM are:
    /// - 2 (-vv): Print logs for all tests.
    /// - 3 (-vvv): Print execution traces for failing tests.
    /// - 4 (-vvvv): Print execution traces for all tests, and setup traces for failing tests.
    /// - 5 (-vvvvv): Print execution and setup traces for all tests, including storage changes.
    #[arg(help_heading = "Display options", global = true, short, long, verbatim_doc_comment, action = ArgAction::Count)]
    verbosity: Verbosity,
}

impl Cli {
    pub fn run(self) -> eyre::Result<()> {
        let runner = CliRunner::default();
        match self.command {
            Commands::OperatorCommands(operator_subcommand) => match operator_subcommand {
                OperatorCommands::BLS(bls_subcommand) => match bls_subcommand {
                    BLSCommands::List(list_command) => {
                        runner.run_command_until_exit(|ctx| list_command.execute(ctx))
                    }
                    BLSCommands::Export(export_command) => {
                        runner.run_command_until_exit(|ctx| export_command.execute(ctx))
                    }
                    BLSCommands::Create(create_command) => {
                        runner.run_command_until_exit(|ctx| create_command.execute(ctx))
                    }
                    BLSCommands::Delete(delete_command) => {
                        runner.run_command_until_exit(|ctx| delete_command.execute(ctx))
                    }
                },
                OperatorCommands::Delete(delete_command) => {
                    runner.run_command_until_exit(|ctx| delete_command.execute(ctx))
                }
                OperatorCommands::Get(get_command) => {
                    runner.run_command_until_exit(|ctx| get_command.execute(ctx))
                }
                OperatorCommands::Import(import_command) => {
                    runner.run_command_until_exit(|ctx| import_command.execute(ctx))
                }
                OperatorCommands::List(list_command) => {
                    runner.run_command_until_exit(|ctx| list_command.execute(ctx))
                }
                OperatorCommands::Register(register_command) => {
                    runner.run_command_until_exit(|ctx| register_command.execute(ctx))
                }
            },
            Commands::VaultCommands(subcommand) => match subcommand {
                VaultCommands::Get(get_command) => {
                    runner.run_command_until_exit(|ctx| get_command.execute(ctx))
                }
                VaultCommands::List(list_command) => {
                    runner.run_command_until_exit(|ctx| list_command.execute(ctx))
                }
                VaultCommands::OptIn(opt_in_command) => {
                    runner.run_command_until_exit(|ctx| opt_in_command.execute(ctx))
                }
                VaultCommands::OptOut(opt_out_command) => {
                    runner.run_command_until_exit(|ctx| opt_out_command.execute(ctx))
                }
            },
            Commands::NetworkCommands(subcommand) => match subcommand {
                // NetworkCommands::Onboard(onboard_command) => {
                //     runner.run_command_until_exit(|ctx| onboard_command.execute(ctx))
                // }
                NetworkCommands::OptIn(opt_in_command) => {
                    runner.run_command_until_exit(|ctx| opt_in_command.execute(ctx))
                }
                NetworkCommands::OptOut(opt_out_command) => {
                    runner.run_command_until_exit(|ctx| opt_out_command.execute(ctx))
                }
                NetworkCommands::PauseKey(pause_key_command) => {
                    runner.run_command_until_exit(|ctx| pause_key_command.execute(ctx))
                }
                NetworkCommands::RegisterKey(register_key_command) => {
                    runner.run_command_until_exit(|ctx| register_key_command.execute(ctx))
                }
                NetworkCommands::RemoveKey(remove_key_command) => {
                    runner.run_command_until_exit(|ctx| remove_key_command.execute(ctx))
                }
                NetworkCommands::Stats(stats_command) => {
                    runner.run_command_until_exit(|ctx| stats_command.execute(ctx))
                }
                NetworkCommands::UnpauseKey(unpause_key_command) => {
                    runner.run_command_until_exit(|ctx| unpause_key_command.execute(ctx))
                }
            },
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(name = "operator", subcommand)]
    OperatorCommands(OperatorCommands),

    #[command(name = "vault", subcommand)]
    VaultCommands(VaultCommands),

    #[command(name = "network", subcommand)]
    NetworkCommands(NetworkCommands),
}

fn get_color_style() -> Styles {
    Styles::styled()
        .usage(AnsiColor::Green.on_default().bold().underline())
        .header(AnsiColor::Yellow.on_default().bold().underline())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}
