use alloy_primitives::{aliases::U96, Address, U256};
use clap::Parser;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use foundry_common::provider::RetryProvider;
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use crate::{
    cmd::utils::{format_number_with_decimals, get_chain_id},
    common::DirsCliArgs,
    symbiotic::{
        calls::{get_delegator_type, get_max_network_limit, is_network, is_vault},
        consts::{get_network_registry, get_vault_factory},
        network_utils::{get_network_metadata, NetworkInfo},
        vault_utils::{get_vault_network_limit_formatted, VaultData, VaultDataTableBuilder},
        DelegatorType,
    },
    utils::{print_loading_until_async, validate_cli_args},
};

use super::utils::get_network_config;

#[derive(Debug, Parser)]
pub struct VaultParametersCommand {
    #[arg(value_name = "ADDRESS", help = "Address of the vault.")]
    vault: Address,

    #[arg(
        value_name = "SUBNETWORK",
        help = "The subnetwork to set the limit for."
    )]
    subnetwork: U96,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl VaultParametersCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            vault,
            subnetwork,
            alias,
            dirs,
            eth,
        } = self;

        validate_cli_args(&eth)?;
        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;

        // Get network config and addresses
        let network_config = get_network_config(chain_id, alias, &dirs)?;
        let network_address = network_config.address;
        let network_registry = get_network_registry(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;

        validate_network_and_vault(
            &provider,
            network_address,
            network_registry,
            vault,
            vault_factory,
        )
        .await?;

        // Fetch vault and network data
        let vault_data = print_loading_until_async(
            "Fetching vault info",
            VaultData::load(chain_id, vault, true, &provider),
        )
        .await?;

        let network_metadata = print_loading_until_async(
            "Fetching network metadata",
            get_network_metadata(network_config.address),
        )
        .await?;

        // Get delegator information
        let delegator = vault_data.delegator.clone().unwrap();
        let delegator_type = get_delegator_type(delegator.clone(), &provider).await?;
        let max_network_limit = get_max_network_limit(
            network_config.address,
            subnetwork,
            delegator.clone(),
            &provider,
        )
        .await?;

        let table = build_table(
            vault_data,
            network_address,
            subnetwork,
            network_metadata,
            max_network_limit,
            delegator_type,
            delegator,
            &provider,
        )
        .await?;

        table.printstd();

        Ok(())
    }
}

async fn validate_network_and_vault(
    provider: &RetryProvider,
    network_address: Address,
    network_registry: Address,
    vault: Address,
    vault_factory: Address,
) -> eyre::Result<()> {
    let is_network = print_loading_until_async(
        "Checking network registration status",
        is_network(network_address, network_registry, provider),
    )
    .await?;

    if !is_network {
        eyre::bail!("Network is not registered");
    }

    let is_vault = print_loading_until_async(
        "Checking vault status",
        is_vault(vault, vault_factory, provider),
    )
    .await?;

    if !is_vault {
        eyre::bail!("Provided address is not a valid Symbiotic vault.");
    }

    Ok(())
}

async fn build_table(
    vault: VaultData,
    network_address: Address,
    subnetwork: U96,
    network_metadata: Option<NetworkInfo>,
    max_network_limit: U256,
    delegator_type: DelegatorType,
    delegator: Address,
    provider: &RetryProvider,
) -> eyre::Result<Table> {
    // Add vault data
    let mut table = VaultDataTableBuilder::from_vault_data(vault.clone())
        .with_name()
        .with_network(network_address, network_metadata)
        .with_subnetwork_identifier(network_address, subnetwork)?
        .with_delegator()
        .with_slasher()
        .with_current_epoch()
        .with_epoch_duration()
        .with_next_epoch_start()
        .with_time_till_next_epoch()
        .build();

    // Add network limits
    let max_limit_formatted =
        format_number_with_decimals(max_network_limit, vault.decimals.unwrap())?;
    let max_limit_display = if max_limit_formatted == "0.000" {
        "-".to_string()
    } else {
        format!(
            "{} ({} {})",
            max_network_limit.to_string(),
            max_limit_formatted,
            vault.symbol.clone().unwrap()
        )
    };
    let vault_network_limit_display = get_vault_network_limit_formatted(
        provider,
        network_address,
        subnetwork,
        &vault,
        delegator,
        delegator_type,
        max_limit_display.clone(),
    )
    .await?;

    table.add_row(row![Fcb -> "Max Network Limit", max_limit_display]);
    table.add_row(row![Fcb -> "Vault Network Limit", vault_network_limit_display]);
    Ok(table)
}
