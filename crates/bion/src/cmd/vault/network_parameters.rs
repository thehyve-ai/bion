use alloy_primitives::{aliases::U96, Address};
use clap::Parser;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    cmd::utils::get_chain_id,
    common::DirsCliArgs,
    symbiotic::{
        calls::{get_max_network_limit, get_network_limit},
        consts::{get_network_registry, get_vault_factory},
        network_utils::{get_network_metadata, validate_network_symbiotic_status},
        vault_utils::{
            validate_vault_symbiotic_status, RowPrefix, VaultData, VaultDataTableBuilder,
        },
    },
    utils::{print_loading_until_async, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Get detailed information for your vault and a network.")]
pub struct NetworkParametersCommand {
    #[arg(value_name = "ADDRESS", help = "Address of the vault.")]
    vault: Address,

    #[arg(value_name = "NETWORK", help = "The network address.")]
    network: Address,

    #[arg(value_name = "SUBNETWORK", help = "The subnetwork identifier.")]
    subnetwork: U96,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl NetworkParametersCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            vault,
            network,
            subnetwork,
            eth,
            ..
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let network_registry = get_network_registry(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;

        validate_network_symbiotic_status(network, network_registry, &provider).await?;
        validate_vault_symbiotic_status(vault, vault_factory, &provider).await?;

        let vault = print_loading_until_async(
            "Fetching vault data",
            VaultData::load(chain_id, vault, true, &provider),
        )
        .await?;

        let network_metadata =
            print_loading_until_async("Fetching network metadata", get_network_metadata(network))
                .await?;

        let Some(delegator) = vault.delegator else {
            eyre::bail!("Invalid vault delegator.");
        };

        let max_network_limit = print_loading_until_async(
            "Fetching max network limit",
            get_max_network_limit(network, subnetwork, delegator, &provider),
        )
        .await?;

        let network_limit = print_loading_until_async(
            "Fetching network limit",
            get_network_limit(network, subnetwork, delegator, &provider),
        )
        .await?;

        let table = VaultDataTableBuilder::from_vault_data(vault)
            .with_name()
            .with_network(network, network_metadata)
            .with_subnetwork_identifier(network, subnetwork)?
            .with_delegator()
            .with_slasher()
            .with_current_epoch()
            .with_epoch_duration()
            .with_next_epoch_start()
            .with_time_till_next_epoch()
            .with_max_network_limit(max_network_limit, RowPrefix::Default)?
            .with_network_limit(network_limit, RowPrefix::Default)?
            .build();
        table.printstd();

        Ok(())
    }
}
