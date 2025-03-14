use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;

use std::time::Instant;

use crate::{
    cmd::utils::get_chain_id,
    symbiotic::{
        consts::get_vault_factory,
        vault_utils::{validate_vault_symbiotic_status, VaultData, VaultDataTableBuilder},
    },
    utils::{print_loading_until_async, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Get information for a Symbiotic vault.")]
pub struct GetVaultCommand {
    #[arg(value_name = "VAULT", help = "The address of the vault.")]
    vault: Address,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl GetVaultCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { vault, eth } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let vault_factory = get_vault_factory(chain_id)?;

        validate_vault_symbiotic_status(vault, vault_factory, &provider).await?;

        let t1 = Instant::now();
        let vault = print_loading_until_async(
            "Loading vault",
            VaultData::load(chain_id, vault, true, &provider),
        )
        .await?;

        println!(
            "\n{}\n",
            format!("Loaded vault in {}ms", t1.elapsed().as_millis()).bright_green()
        );

        let table_builder = VaultDataTableBuilder::from_vault_data(vault);
        let table_builder = table_builder.with_all()?;
        let table = table_builder.build();
        table.printstd();

        Ok(())
    }
}
