use alloy_primitives::{aliases::U96, U256};
use clap::Parser;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;
use itertools::Itertools;
use prettytable::{row, Table};

use crate::{
    cmd::{alias_utils::get_alias_config, utils::get_chain_id},
    common::DirsCliArgs,
    symbiotic::{
        calls::get_network_limit,
        consts::get_network_registry,
        network_utils::validate_network_status,
        vault_utils::{
            fetch_token_datas, fetch_vault_addresses, fetch_vault_datas,
            fetch_vault_symbiotic_metadata,
        },
    },
    utils::{print_loading_until_async, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "List vaults where your network is opted in.")]
pub struct ListVaultsCommand {
    #[arg(long, help = "Only show verified vaults.", default_value = "false")]
    verified_only: bool,

    #[arg(value_name = "SUBNETWORK", help = "The index of the subnetwork.")]
    subnetwork: U96,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl ListVaultsCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            verified_only,
            subnetwork,
            alias,
            dirs,
            eth,
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let network_config = get_alias_config(chain_id, alias, &dirs)?;
        let network = network_config.address;
        let network_registry = get_network_registry(chain_id)?;

        validate_network_status(network, network_registry, &provider).await?;

        let vault_addresses = fetch_vault_addresses(&provider, chain_id).await?;
        let vaults = fetch_vault_datas(&provider, chain_id, vault_addresses).await?;
        let mut valid_vaults = Vec::new();
        for vault in vaults {
            let Some(delegator) = vault.delegator else {
                continue;
            };

            let network_limit = print_loading_until_async(
                "Fetching vault network limit",
                get_network_limit(network, subnetwork, delegator, &provider),
            )
            .await;

            if let Ok(network_limit) = network_limit {
                if network_limit > U256::ZERO {
                    valid_vaults.push(vault);
                }
            }
        }

        if valid_vaults.is_empty() {
            eyre::bail!("No vaults found for the network");
        }

        let valid_vaults = fetch_vault_symbiotic_metadata(valid_vaults).await?;
        let valid_vaults = fetch_token_datas(&provider, chain_id, valid_vaults).await?;

        let mut table = Table::new();

        // table headers
        table.add_row(row![
            b -> "#",
            b -> "name",
            b -> "address",
            b -> "collateral_token",
            b -> "tvl",
            b -> "delegated"
        ]);

        let mut i = 0;
        for vault in valid_vaults
            .into_iter()
            .sorted_by(|a, b| b.active_stake.cmp(&a.active_stake))
        {
            let vault_address = vault.address;
            let name = vault.symbiotic_metadata.clone().map(|m| m.name);
            if verified_only && name.is_none() {
                continue;
            }

            let symbiotic_link = format!(
                "\x1B]8;;https://app.symbiotic.fi/vault/{}\x1B\\{}\x1B]8;;\x1B\\",
                vault_address,
                name.unwrap_or("Unverified vault".to_string())
            );
            let vault_link: String = format!(
                "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
                vault_address, vault_address
            );

            let collateral_link = format!(
                "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
                vault.collateral.unwrap(),
                vault.symbol.as_ref().unwrap()
            );

            let row = row![
                i + 1,
                symbiotic_link,
                vault_link,
                collateral_link,
                vault.total_stake_formatted().unwrap(),
                vault.active_stake_formatted_with_percentage().unwrap(),
            ];

            table.add_row(row);
            i += 1;
        }

        table.printstd();

        Ok(())
    }
}
