use bion::cmd::{
    add_alias::AddAliasCommand, bls::BLSCommands, hyve::HyveCommand,
    list_aliases::ListAliasesCommand, network::NetworkCommand, operator::OperatorCommand,
    remove_alias::RemoveAliasCommand, symbiotic::SymbioticCommands, vault::VaultCommand,
};
use clap::{
    builder::{styling::AnsiColor, Styles},
    ArgAction, Parser, Subcommand,
};
use hyve_cli_runner::CliRunner;
use hyve_version::SHORT_VERSION;

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
            Commands::AddAlias(add_alias) => {
                runner.run_command_until_exit(|ctx| add_alias.execute(ctx))
            }
            Commands::ListAliases(list_aliases) => {
                runner.run_command_until_exit(|ctx| list_aliases.execute(ctx))
            }
            Commands::RemoveAlias(remove_alias) => {
                runner.run_command_until_exit(|ctx| remove_alias.execute(ctx))
            }
            Commands::BLS(bls) => match bls {
                BLSCommands::Create(create) => {
                    runner.run_command_until_exit(|ctx| create.execute(ctx))
                }
                BLSCommands::Delete(delete) => {
                    runner.run_command_until_exit(|ctx| delete.execute(ctx))
                }
                BLSCommands::Export(export) => {
                    runner.run_command_until_exit(|ctx| export.execute(ctx))
                }
                BLSCommands::List(list) => runner.run_command_until_exit(|ctx| list.execute(ctx)),
            },
            Commands::Hyve(hyve) => runner.run_command_until_exit(|ctx| hyve.execute(ctx)),
            Commands::Symbiotic(symbiotic) => match symbiotic {
                SymbioticCommands::GetVault(get_vault) => {
                    runner.run_command_until_exit(|ctx| get_vault.execute(ctx))
                }
                SymbioticCommands::ListVaults(list_vaults) => {
                    runner.run_command_until_exit(|ctx| list_vaults.execute(ctx))
                }
            },
            Commands::Network(network) => runner.run_command_until_exit(|ctx| network.execute(ctx)),
            Commands::Operator(operator) => {
                runner.run_command_until_exit(|ctx| operator.execute(ctx))
            }
            Commands::Vault(vault) => runner.run_command_until_exit(|ctx| vault.execute(ctx)),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(name = "add-alias")]
    AddAlias(AddAliasCommand),

    #[command(name = "list-aliases")]
    ListAliases(ListAliasesCommand),

    #[command(name = "remove-alias")]
    RemoveAlias(RemoveAliasCommand),

    #[command(name = "bls", subcommand)]
    BLS(BLSCommands),

    #[command(name = "hyve")]
    Hyve(HyveCommand),

    #[command(name = "symbiotic", subcommand)]
    Symbiotic(SymbioticCommands),

    #[command(name = "network")]
    Network(NetworkCommand),

    #[command(name = "operator")]
    Operator(OperatorCommand),

    #[command(name = "vault")]
    Vault(VaultCommand),
}

fn get_color_style() -> Styles {
    Styles::styled()
        .usage(AnsiColor::Green.on_default().bold().underline())
        .header(AnsiColor::Yellow.on_default().bold().underline())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}
