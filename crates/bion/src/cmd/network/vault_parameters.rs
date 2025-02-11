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
        calls::{
            get_delegator_type, get_max_network_limit, get_network_limit, get_vault_collateral,
            get_vault_delegator, is_network, is_vault,
        },
        consts::{get_network_registry, get_vault_factory},
        network_utils::{get_network_metadata, NetworkInfo},
        utils::{get_network_link, get_subnetwork, get_vault_link},
        vault_utils::{
            fetch_token_data, get_vault_metadata, VaultData, VaultDataTableBuilder, VaultInfo,
        },
        DelegatorType,
    },
    utils::{print_error_message, print_loading_until_async, validate_cli_args},
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
    pub subnetwork: U96,

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
            VaultData::load(vault, &provider, chain_id),
        )
        .await?;

        let (vault_metadata, network_metadata) = tokio::join!(
            print_loading_until_async(
                "Fetching vault metadata",
                get_vault_metadata(vault_data.address)
            ),
            print_loading_until_async(
                "Fetching network metadata",
                get_network_metadata(network_config.address)
            )
        );
        let vault_metadata = vault_metadata?;
        let network_metadata = network_metadata?;

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
            vault_metadata,
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
        print_error_message("Network is not registered");
        return Ok(());
    }

    let is_vault = print_loading_until_async(
        "Checking vault status",
        is_vault(vault, vault_factory, provider),
    )
    .await?;

    if !is_vault {
        print_error_message("Provided address is not a valid Symbiotic vault.");
        return Ok(());
    }

    Ok(())
}

async fn build_table(
    vault: VaultData,
    network_address: Address,
    subnetwork: U96,
    vault_metadata: Option<VaultInfo>,
    network_metadata: Option<NetworkInfo>,
    max_network_limit: U256,
    delegator_type: DelegatorType,
    delegator: Address,
    provider: &RetryProvider,
) -> eyre::Result<Table> {
    let mut table = Table::new();

    // Add vault and network information
    let vault_link = get_vault_link(
        vault.address,
        vault_metadata
            .map(|v| v.name)
            .unwrap_or("UNVERIFIED".to_string()),
    );
    let network_link = get_network_link(
        network_address,
        network_metadata
            .map(|v| v.name)
            .unwrap_or("UNVERIFIED".to_string()),
    );

    table.add_row(row![Fcb -> "Vault", vault_link]);
    table.add_row(row![Fcb -> "Network", network_link]);
    table.add_row(row![Fcb -> "Subnetwork Identifier",
        get_subnetwork(network_address, subnetwork)?.to_string()
    ]);

    // Add vault data
    table = VaultDataTableBuilder::from_vault_data(vault.clone())
        .with_table(table)
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
        network_address,
        subnetwork,
        delegator,
        provider,
        &vault,
        delegator_type,
        max_limit_display.clone(),
    )
    .await?;

    table.add_row(row![Fcb -> "Max Network Limit", max_limit_display]);
    table.add_row(row![Fcb -> "Vault Network Limit", vault_network_limit_display]);
    Ok(table)
}

async fn get_vault_network_limit_formatted(
    network: Address,
    subnetwork: U96,
    delegator: Address,
    provider: &RetryProvider,
    vault: &VaultData,
    delegator_type: DelegatorType,
    max_network_limit: String,
) -> eyre::Result<String> {
    // Add vault network limit based on delegator type
    let vault_limit_display = if delegator_type == DelegatorType::OperatorNetworkSpecificDelegator {
        max_network_limit
    } else {
        let network_limit = get_network_limit(network, subnetwork, delegator, provider).await?;
        let network_limit_formatted =
            format_number_with_decimals(network_limit, vault.decimals.clone().unwrap())?;
        if network_limit_formatted == "0.000" {
            format!(
                "{} (- {})",
                network_limit.to_string(),
                vault.symbol.clone().unwrap()
            )
        } else {
            format!(
                "{} ({} {})",
                network_limit.to_string(),
                network_limit_formatted,
                vault.symbol.clone().unwrap()
            )
        }
    };
    Ok(vault_limit_display)
}
