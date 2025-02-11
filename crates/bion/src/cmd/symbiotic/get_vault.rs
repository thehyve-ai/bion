use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use std::time::Instant;

use crate::{
    cmd::utils::get_chain_id,
    symbiotic::{
        calls::is_vault,
        consts::get_vault_factory,
        utils::get_vault_link,
        vault_utils::{
            fetch_token_datas, fetch_vault_datas, fetch_vault_extra_metadata, get_vault_metadata,
            VaultData, VaultDataTableBuilder,
        },
    },
    utils::{parse_duration_secs, parse_epoch_ts, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Get information for Symbiotic vault.")]
pub struct GetVaultCommand {
    #[arg(value_name = "VAULT_ADDRESS", help = "The address of the vault.")]
    vault_address: Address,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl GetVaultCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { vault_address, eth } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        {
            let txt = format!("Loading vault {} on chain {}.", vault_address, chain_id,);
            println!("{}", txt.as_str().bright_cyan());
        }

        let vault_factory = get_vault_factory(chain_id)?;

        let is_vault = is_vault(vault_address, vault_factory, &provider).await?;
        if !is_vault {
            return Err(eyre::eyre!("Address is not a vault."));
        }

        let t1 = Instant::now();

        let vault = VaultData::load(vault_address, &provider, chain_id).await?;

        {
            let txt = format!("Loaded vault in {}ms", t1.elapsed().as_millis());
            println!("{}", txt.as_str().bright_green());
        }

        let table_builder = VaultDataTableBuilder::from_vault_data(vault);
        let table_builder = table_builder.with_all().await?;
        let table = table_builder.build();
        table.printstd();

        Ok(())
    }
}
