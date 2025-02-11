use alloy_primitives::{aliases::U96, Address, U256};
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    cmd::utils::get_chain_id,
    common::DirsCliArgs,
    symbiotic::{
        calls::{get_delegator_type, get_vault_delegator, is_opted_in_vault},
        consts::{get_network_registry, get_vault_factory, get_vault_opt_in_service},
        network_utils::validate_network_status,
        utils::get_subnetwork,
        vault_utils::validate_vault_status,
        DelegatorType,
    },
    utils::{print_error_message, print_loading_until_async, validate_cli_args},
};

use super::utils::{get_vault_admin_config, set_foundry_signing_method};

#[derive(Debug, Parser)]
pub struct SetOperatorNetworkSharesCommand {
    #[arg(value_name = "ADDRESS", help = "The address of the network.")]
    network: Address,

    #[arg(
        value_name = "SUBNETWORK",
        help = "The subnetwork to set the limit for."
    )]
    subnetwork: U96,

    #[arg(value_name = "OPERATOR", help = "Address of the operator.")]
    operator: Address,

    #[arg(value_name = "VAULT", help = "The address of the vault.")]
    vault: Address,

    #[arg(value_name = "SHARES", help = "The shares to set.")]
    shares: U256,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    tx: TransactionOpts,

    #[clap(flatten)]
    eth: EthereumOpts,

    /// Send via `eth_sendTransaction using the `--from` argument or $ETH_FROM as sender
    #[arg(long, requires = "from")]
    pub unlocked: bool,

    /// Timeout for sending the transaction.
    #[arg(long, env = "ETH_TIMEOUT")]
    pub timeout: Option<u64>,

    /// The number of confirmations until the receipt is fetched.
    #[arg(long, default_value = "1")]
    confirmations: u64,
}

impl SetOperatorNetworkSharesCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            network,
            subnetwork,
            operator,
            vault,
            shares,
            alias,
            dirs,
            mut eth,
            tx,
            confirmations,
            timeout,
            unlocked,
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let network_registry = get_network_registry(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;
        let vault_opt_in_service = get_vault_opt_in_service(chain_id)?;
        let vault_admin_config = get_vault_admin_config(chain_id, alias, &dirs)?;
        set_foundry_signing_method(&vault_admin_config, &mut eth)?;

        validate_network_status(network, network_registry, &provider).await?;
        validate_vault_status(vault, vault_factory, &provider).await?;

        let is_opted_in = print_loading_until_async(
            "Checking network opt in status in vault",
            is_opted_in_vault(operator, vault, vault_opt_in_service, &provider),
        )
        .await?;

        if !is_opted_in {
            print_error_message("Operator is not opted in vault.");
            return Ok(());
        }

        let delegator =
            print_loading_until_async("Fetching delegator", get_vault_delegator(vault, &provider))
                .await?;

        let delegator_type = print_loading_until_async(
            "Fetching delegator type",
            get_delegator_type(delegator, &provider),
        )
        .await?;

        if delegator_type != DelegatorType::NetworkRestakeDelegator {
            print_error_message(
                "Operator Network shares can only be set for NetworkRestakeDelegator.",
            );
            return Ok(());
        }

        let subnetwork_address = get_subnetwork(network, subnetwork)?;

        Ok(())
    }
}
