use alloy_primitives::{aliases::U96, Address, U256};
use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_common::ens::NameOrAddress;
use hyve_cli_runner::CliContext;
use safe_multisig::SafeClient;

use crate::{
    cast::{cmd::send::SendTxArgs, utils::build_tx},
    cmd::{
        alias_utils::{get_alias_config, set_foundry_signing_method},
        utils::get_chain_id,
    },
    common::{DirsCliArgs, SigningMethod},
    symbiotic::{
        calls::{get_delegator_type, get_max_network_limit, get_network_limit},
        consts::{get_network_registry, get_vault_factory},
        network_utils::{get_network_metadata, validate_network_symbiotic_status},
        vault_utils::{
            validate_vault_symbiotic_status, RowPrefix, VaultData, VaultDataTableBuilder,
        },
        DelegatorType,
    },
    utils::{print_loading_until_async, read_user_confirmation, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Set a max network limit on specific vault for your network.")]
pub struct SetMaxNetworkLimitCommand {
    #[arg(value_name = "VAULT", help = "The address of the vault.")]
    vault: Address,

    #[arg(value_name = "SUBNETWORK", help = "The index of the subnetwork.")]
    subnetwork: U96,

    #[arg(value_name = "LIMIT", help = "The limit to set.")]
    limit: U256,

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
            subnetwork,
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
        let network_config = get_alias_config(chain_id, alias, &dirs)?;
        let network = network_config.address;
        let network_registry = get_network_registry(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;
        // set_foundry_signing_method(&network_config, &mut eth)?;

        validate_network_symbiotic_status(network, network_registry, &provider).await?;
        validate_vault_symbiotic_status(vault, vault_factory, &provider).await?;

        let vault = print_loading_until_async(
            "Fetching vault data",
            VaultData::load(chain_id, vault, false, &provider),
        )
        .await?;

        let Some(collateral_decimals) = vault.decimals else {
            eyre::bail!("Invalid vault collateral.");
        };

        let Some(delegator) = vault.delegator else {
            eyre::bail!("Invalid vault delegator.");
        };

        let delegator_type = print_loading_until_async(
            "Fetching delgator type",
            get_delegator_type(delegator, &provider),
        )
        .await?;

        if delegator_type == DelegatorType::OperatorNetworkSpecificDelegator {
            eyre::bail!(
                "Max Network limit cannot be set for vaults with an OperatorNetworkSpecificDelegator.",
            );
        }

        let limit = limit * U256::from(10).pow(U256::from(collateral_decimals));

        let to = NameOrAddress::Address(delegator);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("setMaxNetworkLimit(uint96 identifier, uint256 amount)".to_string()),
            args: vec![subnetwork.to_string(), limit.to_string()],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth: eth.clone(),
            path: None,
        };

        println!("\n{}", "Increasing max network limit".bright_cyan());

        let old_max_network_limit = print_loading_until_async(
            "Fetching max network limit",
            get_max_network_limit(network, subnetwork, delegator, &provider),
        )
        .await?;

        if old_max_network_limit == limit {
            eyre::bail!("New limit is the same as current limit.");
        }

        let network_limit = print_loading_until_async(
            "Fetching network limit",
            get_network_limit(network, subnetwork, delegator, &provider),
        )
        .await?;

        let network_metadata =
            print_loading_until_async("Fetching network metadata", get_network_metadata(network))
                .await?;
        let table = VaultDataTableBuilder::from_vault_data(vault)
            .with_name()
            .with_network(network, network_metadata)
            .with_subnetwork_identifier(network, subnetwork)?
            .with_max_network_limit(old_max_network_limit, RowPrefix::Old)?
            .with_max_network_limit(limit, RowPrefix::New)?
            .with_network_limit(network_limit, RowPrefix::Default)?
            .build();
        table.printstd();

        println!("\n{}", "Do you wish to continue? (y/n)".bright_cyan());

        let confirmation: String = read_user_confirmation()?;
        if confirmation.trim().to_lowercase().as_str() == "n"
            || confirmation.trim().to_lowercase().as_str() == "no"
        {
            eyre::bail!("Exiting...");
        }

        match network_config.signing_method {
            Some(SigningMethod::MultiSig) => {
                let safe = SafeClient::new(chain_id)?;
                eth.wallet.from = Some(network_config.address);
                let config = eth.load_config()?;
                let tx = build_tx(arg, &config, &provider).await?;
                set_foundry_signing_method(&network_config, &mut eth)?;
                let signer = eth.wallet.signer().await?;
                safe.send_tx(network, signer, tx, &provider).await?;
            }
            _ => {
                set_foundry_signing_method(&network_config, &mut eth)?;
                let _ = arg.run().await?;
            }
        };
        Ok(())
    }
}
