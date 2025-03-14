use alloy_primitives::{aliases::U96, hex::ToHexExt, Address, U256};
use alloy_sol_types::SolValue;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_common::ens::NameOrAddress;
use hyve_cli_runner::CliContext;
use safe_multisig::{transaction_data::ExecutableSafeTransaction, SafeClient};

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::{
        alias_utils::{get_alias_config, set_foundry_signing_method},
        utils::get_chain_id,
    },
    common::{DirsCliArgs, SigningMethod},
    symbiotic::{
        calls::{
            get_delegator_type, get_max_network_limit, get_network_limit,
            get_operator_network_shares, get_total_operator_network_shares,
        },
        consts::{
            get_network_opt_in_service, get_network_registry, get_operator_registry,
            get_vault_factory, get_vault_opt_in_service,
        },
        network_utils::{
            get_network_metadata, validate_network_opt_in_status, validate_network_symbiotic_status,
        },
        operator_utils::validate_operator_symbiotic_status,
        utils::get_subnetwork,
        vault_utils::{
            validate_vault_opt_in_status, validate_vault_symbiotic_status, RowPrefix, VaultData,
            VaultDataTableBuilder,
        },
        DelegatorType,
    },
    utils::{print_loading_until_async, read_user_confirmation, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(
    about = "Set operator operator network shares for an operator within a network in your vault."
)]
pub struct SetOperatorNetworkSharesCommand {
    #[arg(value_name = "ADDRESS", help = "The address of the network.")]
    network: Address,

    #[arg(value_name = "SUBNETWORK", help = "The subnetwork to set the limit for.")]
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
        let vault_admin_config = get_alias_config(chain_id, alias, &dirs)?;
        let network_opt_in_service = get_network_opt_in_service(chain_id)?;
        let network_registry = get_network_registry(chain_id)?;
        let operator_registry = get_operator_registry(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;
        let vault_opt_in_service = get_vault_opt_in_service(chain_id)?;
        set_foundry_signing_method(&vault_admin_config, &mut eth)?;

        validate_operator_symbiotic_status(operator, operator_registry, &provider).await?;
        validate_network_symbiotic_status(network, network_registry, &provider).await?;
        validate_network_opt_in_status(operator, network, network_opt_in_service, &provider)
            .await?;
        validate_vault_symbiotic_status(vault, vault_factory, &provider).await?;
        validate_vault_opt_in_status(operator, vault, vault_opt_in_service, &provider).await?;

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

        if delegator_type != DelegatorType::NetworkRestakeDelegator {
            eyre::bail!("Operator Network shares can only be set for NetworkRestakeDelegator.",);
        }

        let shares = shares * U256::from(10).pow(U256::from(collateral_decimals));
        let subnetwork_address = get_subnetwork(network, subnetwork)?;

        let to = NameOrAddress::Address(delegator);

        let args = SendTxArgs {
            to: Some(to),
            sig: Some(
                "setOperatorNetworkShares(bytes32 subnetwork, address operator, uint256 shares)"
                    .to_string(),
            ),
            args: vec![
                subnetwork_address.abi_encode().encode_hex_upper_with_prefix(),
                operator.to_string(),
                shares.to_string(),
            ],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth: eth.clone(),
            path: None,
        };

        println!("\n{}", "Setting operator network shares".bright_cyan());

        let max_network_limit = print_loading_until_async(
            "Fetching max network limit",
            get_max_network_limit(network, subnetwork, delegator, &provider),
        )
        .await?;

        if max_network_limit > U256::ZERO && max_network_limit < shares {
            eyre::bail!("Cannot set operator shares higher than the max network limit.");
        }

        let network_limit = print_loading_until_async(
            "Fetching network limit",
            get_network_limit(network, subnetwork, delegator, &provider),
        )
        .await?;

        let old_operator_network_shares = print_loading_until_async(
            "Fetching operator network shares",
            get_operator_network_shares(network, subnetwork, operator, delegator, &provider),
        )
        .await?;

        if old_operator_network_shares == shares {
            eyre::bail!("New shares are the same as current shares.");
        }

        let total_operator_network_shares = print_loading_until_async(
            "Fetching total operator network shares",
            get_total_operator_network_shares(network, subnetwork, delegator, &provider),
        )
        .await?;

        let network_metadata =
            print_loading_until_async("Fetching network metadata", get_network_metadata(network))
                .await?;
        let table = VaultDataTableBuilder::from_vault_data(vault)
            .with_name()
            .with_network(network, network_metadata)
            .with_subnetwork_identifier(network, subnetwork)?
            .with_max_network_limit(max_network_limit, RowPrefix::Default)?
            .with_network_limit(network_limit, RowPrefix::Default)?
            .with_operator_network_shares(old_operator_network_shares, RowPrefix::Old)?
            .with_operator_network_shares(shares, RowPrefix::New)?
            .with_total_operator_network_shares(total_operator_network_shares)?
            .build();
        table.printstd();

        println!("\n{}", "Do you wish to continue? (y/n)".bright_cyan());

        let confirmation: String = read_user_confirmation()?;
        if confirmation.trim().to_lowercase().as_str() == "n"
            || confirmation.trim().to_lowercase().as_str() == "no"
        {
            eyre::bail!("Exiting...");
        }

        match vault_admin_config.signing_method {
            Some(SigningMethod::MultiSig) => {
                let safe = SafeClient::new(chain_id)?;
                let signer = eth.wallet.signer().await?;
                let mut executable_args = args.clone();
                if let Some(ExecutableSafeTransaction { safe_address, input_data }) = safe
                    .send_tx(vault_admin_config.address, signer, args.try_into()?, &provider)
                    .await?
                {
                    executable_args.to = Some(NameOrAddress::Address(safe_address));
                    executable_args.sig = Some(input_data);
                    let _ = executable_args.run().await?;
                }
            }
            _ => {
                let _ = args.run().await?;
            }
        };
        Ok(())
    }
}
