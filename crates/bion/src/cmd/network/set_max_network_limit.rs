use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_common::ens::NameOrAddress;
use hyve_cli_runner::CliContext;

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::utils::get_chain_id,
    common::DirsCliArgs,
    symbiotic::{
        calls::{get_vault_delegator, is_network, is_vault},
        consts::{get_network_registry, get_vault_factory},
    },
    utils::{
        print_error_message, print_loading_until_async, print_success_message, validate_cli_args,
    },
};

use super::utils::{get_network_config, set_foundry_signing_method};

#[derive(Debug, Parser)]
#[clap(about = "Set a max network limit on specific vault for your network.")]
pub struct SetMaxNetworkLimitCommand {
    #[arg(value_name = "VAULT", help = "The address of the vault.")]
    pub vault: Address,

    #[arg(value_name = "SUBNET", help = "The subnet to set the limit for.")]
    pub subnet: usize,

    #[arg(value_name = "LIMIT", help = "The limit to set.")]
    pub limit: usize,

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

impl SetMaxNetworkLimitCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            vault,
            subnet,
            limit,
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
        let network_config = get_network_config(chain_id, alias, &dirs)?;
        set_foundry_signing_method(&network_config, &mut eth)?;

        let network_registry = get_network_registry(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;

        let is_network = print_loading_until_async(
            "Checking network registration status",
            is_network(network_config.address, network_registry, &provider),
        )
        .await?;

        if !is_network {
            print_error_message("Network is not registered");
            return Ok(());
        }

        let is_vault = print_loading_until_async(
            "Checking vault status",
            is_vault(vault, vault_factory, &provider),
        )
        .await?;

        if !is_vault {
            print_error_message("Provided address is not a valid Symbiotic vault.");
            return Ok(());
        }

        let delegator =
            print_loading_until_async("Fetching delegator", get_vault_delegator(vault, &provider))
                .await?;

        let to = NameOrAddress::Address(delegator);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("setMaxNetworkLimit(uint96 identifier, uint256 amount)".to_string()),
            args: vec![subnet.to_string(), limit.to_string()],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth,
            path: None,
        };
        if let Ok(..) = arg.run().await {
            print_success_message("✅ Successfully set max network limit.");
        } else {
            print_error_message("❌ Failed to set max network limit, please try again.");
        }
        Ok(())
    }
}
