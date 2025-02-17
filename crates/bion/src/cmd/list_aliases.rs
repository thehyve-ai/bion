use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use crate::{common::DirsCliArgs, utils::validate_cli_args};

use super::{alias_utils::get_or_create_alias_definitions, utils::get_chain_id};

#[derive(Debug, Parser)]
#[clap(about = "List all aliases.")]
pub struct ListAliasesCommand {
    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl ListAliasesCommand {
    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self { dirs, eth } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;

        let definitions = get_or_create_alias_definitions(chain_id, &dirs)?;

        if definitions.is_empty() {
            println!("{}", "No aliases found.".bright_cyan().bold());
            return Ok(());
        }

        let mut table = Table::new();
        table.add_row(row!["Alias", "Address"]);

        for (alias, address) in definitions.iter() {
            table.add_row(row![alias, address,]);
        }

        table.printstd();

        Ok(())
    }
}
