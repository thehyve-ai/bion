use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use std::time::Instant;

use crate::{
    cmd::utils::get_chain_id,
    symbiotic::{
        calls::is_vault,
        consts::get_vault_factory,
        vault_utils::{
            fetch_token_datas, fetch_vault_datas, fetch_vault_extra_metadata, get_vault_metadata,
            VaultData,
        },
    },
    utils::{parse_duration_secs, parse_epoch_ts, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Get information for Symbiotic vault.")]
pub struct GetVaultCommand {
    #[arg(value_name = "VAULT_ADDRESS", help = "The address of the vault.")]
    vault_address: Address,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl GetVaultCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { vault_address, eth } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        {
            let txt = format!("Loading vault {} on chain {}.", vault_address, chain_id,);
            println!("{}", txt.as_str().bright_cyan());
        }

        let vault_factory = get_vault_factory(chain_id)?;

        let is_vault = is_vault(vault_address, vault_factory, &provider).await?;
        if !is_vault {
            return Err(eyre::eyre!("Address is not a vault."));
        }

        let t1 = Instant::now();
        let vaults = fetch_vault_datas(&provider, chain_id, vec![vault_address]).await?;
        let vaults = fetch_vault_extra_metadata(&provider, chain_id, vaults).await?;
        let vaults = fetch_token_datas(&provider, chain_id, vaults).await?;

        if vaults.is_empty() {
            println!(
                "{}",
                "🤔 what did you mess up in your vault config? Couldn't find it.".bright_red()
            );
            return Ok(());
        }

        {
            let txt = format!("Loaded vault in {}ms", t1.elapsed().as_millis());
            println!("{}", txt.as_str().bright_green());
        }

        let vault: VaultData = vaults.first().unwrap().clone();
        let vault_metadata = get_vault_metadata(vault.address).await?;
        let mut table = Table::new();

        let symbiotic_link = format!(
            "\x1B]8;;https://app.symbiotic.fi/vault/{}\x1B\\{}\x1B]8;;\x1B\\",
            vault_address,
            vault_metadata
                .map(|v| v.name)
                .unwrap_or("Unverified vault".to_string())
        );
        table.add_row(row![
            Fcb -> "Name",
            symbiotic_link
        ]);

        let vault_link: String = format!(
            "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
            vault_address, vault_address
        );
        table.add_row(row![Fcb ->"Address",  vault_link]);

        let collateral_link = format!(
            "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
            vault.collateral.unwrap(),
            vault.symbol.clone().unwrap()
        );
        table.add_row(row![Fcb -> "Collateral",   collateral_link]);
        let delegator_link = format!(
            "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
            vault.delegator.unwrap(),
            vault.delegator.unwrap()
        );
        table.add_row(row![Fcb -> "Delegator", delegator_link]);

        let slasher_link = format!(
            "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
            vault.slasher.unwrap(),
            vault.slasher.unwrap()
        );
        table.add_row(row![Fcb -> "Slasher",  slasher_link]);

        let burner_link = format!(
            "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
            vault.burner.unwrap(),
            vault.burner.unwrap()
        );
        table.add_row(row![Fcb -> "Burner",  burner_link]);

        let mut deposit_limit = vault.deposit_limit_formatted().unwrap();
        if deposit_limit == "0.000" {
            deposit_limit = "-".to_string();
        }
        table.add_row(row![Fcb -> "Deposit limit",  deposit_limit]);

        let deposit_whitelist = match vault.deposit_whitelist.unwrap() {
            true => "✅",
            false => "❌",
        };
        table.add_row(row![Fcb -> "Deposit whitelist",  deposit_whitelist]);
        table.add_row(row![Fcb -> "Total Stake",  vault.total_stake_formatted().unwrap()]);
        table.add_row(row![
            Fcb -> "Active Stake",
             vault.active_stake_formatted_with_percentage().unwrap()
        ]);
        table.add_row(row![Fcb -> "Current epoch",  vault.current_epoch.unwrap()]);
        table.add_row(
            row![Fcb -> "Current epoch start", parse_epoch_ts(vault.current_epoch_start.unwrap())],
        );
        table.add_row(
            row![Fcb -> "Epoch duration",  parse_duration_secs(vault.epoch_duration.unwrap())],
        );
        table.add_row(
            row![Fcb -> "Next epoch start", parse_epoch_ts(vault.next_epoch_start.unwrap())],
        );

        table.printstd();

        Ok(())
    }
}
