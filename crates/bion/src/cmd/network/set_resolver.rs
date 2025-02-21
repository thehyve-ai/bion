use alloy_primitives::{aliases::U96, Address};
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_common::ens::NameOrAddress;
use hyve_cli_runner::CliContext;

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::{
        alias_utils::{get_alias_config, set_foundry_signing_method},
        utils::get_chain_id,
    },
    common::DirsCliArgs,
    symbiotic::{
        calls::{get_slasher_type, get_vault_slasher},
        consts::{get_network_registry, get_vault_factory},
        network_utils::validate_network_status,
        vault_utils::validate_vault_status,
        SlasherType,
    },
    utils::{print_loading_until_async, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Set the resolver for your network.")]
pub struct SetResolverCommand {
    #[arg(value_name = "VAULT", help = "The address of the vault.")]
    vault: Address,

    #[arg(value_name = "SUBNETWORK", help = "The index of the subnetwork.")]
    subnetwork: U96,

    #[arg(value_name = "RESOLVER", help = "The address of the resolver.")]
    resolver: Address,

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

impl SetResolverCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            vault,
            subnetwork,
            resolver,
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
        let network_config = get_alias_config(chain_id, alias, &dirs)?;
        let network = network_config.address;
        let network_registry = get_network_registry(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;
        set_foundry_signing_method(&network_config, &mut eth)?;

        validate_network_status(network, network_registry, &provider).await?;
        validate_vault_status(vault, vault_factory, &provider).await?;

        let slasher = print_loading_until_async(
            "Fetching vault slasher",
            get_vault_slasher(vault, &provider),
        )
        .await?;

        let slasher_type = print_loading_until_async(
            "Fetching slasher type",
            get_slasher_type(slasher, &provider),
        )
        .await?;

        if slasher_type != SlasherType::VetoSlasher {
            eyre::bail!("Only vaults with a veto slasher can set a resolver.");
        }

        let to = NameOrAddress::Address(slasher);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("setResolver(uint96 identifier, address resolver, bytes hints)".to_string()),
            args: vec![
                subnetwork.to_string(),
                resolver.to_string(),
                "0x".to_string(),
            ],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth,
            path: None,
        };

        let _ = arg.run().await?;
        Ok(())
    }
}
