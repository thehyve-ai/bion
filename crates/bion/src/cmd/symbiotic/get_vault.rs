use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use std::{str::FromStr, time::Instant};

use crate::{
    cmd::utils::get_chain_id,
    common::consts::TESTNET_VAULTS,
    symbiotic::{
        calls::{get_vault_active_stake, get_vault_total_stake},
        utils::{fetch_token_datas, fetch_vault_datas, get_vault_metadata, VaultData},
    },
    utils::validate_cli_args,
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
        validate_cli_args(None, &self.eth).await?;

        let config = self.eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        {
            let txt = format!(
                "Loading vault {} on chain {}.",
                self.vault_address, chain_id,
            );
            println!("{}", txt.as_str().bright_cyan());
        }

        let t1 = Instant::now();
        let vaults = fetch_vault_datas(&provider, chain_id, vec![self.vault_address]).await?;
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

        table.add_row(row![
            b -> "Name",
            vault_metadata
                .map(|v| v.name)
                .unwrap_or("Unverified vault".to_string())
        ]);
        table.add_row(row![b ->"Address", vault.address]);

        let collateral = format!(
            "{} ({})",
            vault.collateral.unwrap(),
            vault.symbol.clone().unwrap()
        );
        table.add_row(row![b -> "Collateral", collateral]);
        table.add_row(row![b -> "Slasher", "todo"]);
        table.add_row(row![b -> "Total Stake", vault.total_stake_formatted().unwrap()]);
        table.add_row(row![
            b -> "Active Stake",
            vault.active_stake_formatted_with_percentage().unwrap()
        ]);

        // let mut i = 0;
        // for vault in vaults
        //     .into_iter()
        //     .sorted_by(|a, b| b.active_stake.cmp(&a.active_stake))
        // {
        //     let vault_address = vault.address;
        //     let url = format!("{SYMBIOTIC_GITHUB_URL}/{vault_address}/{VAULT_FILE_NAME}",);
        //     let res = reqwest::get(&url).await?;

        //     let name = match res.error_for_status() {
        //         Ok(response) => {
        //             let vault_info: VaultInfo = serde_json::from_str(&response.text().await?)?;
        //             Some(vault_info.name)
        //         }
        //         _ => {
        //             if self.verified_only {
        //                 continue;
        //             }
        //             None
        //         }
        //     };

        //     let total_stake_formatted =
        //         format_number_with_decimals(vault.total_stake.unwrap(), vault.decimals.unwrap())?;

        //     let active_stake_formatted = {
        //         let active = format_number_with_decimals(
        //             vault.active_stake.unwrap(),
        //             vault.decimals.unwrap(),
        //         )?;
        //         let total_f64 = f64::from_str(&total_stake_formatted).unwrap();
        //         let active_f64 = f64::from_str(&active).unwrap();
        //         let percentage = if total_f64 > 0.0 {
        //             (active_f64 / total_f64 * 100.0).round()
        //         } else {
        //             0.0
        //         };
        //         format!("{} ({:.0}%)", active, percentage)
        //     };

        //     let symbiotic_link = format!(
        //         "\x1B]8;;https://app.symbiotic.fi/vault/{}\x1B\\{}\x1B]8;;\x1B\\",
        //         vault_address,
        //         name.unwrap_or("Unverified vault".to_string())
        //     );
        //     let vault_link: String = format!(
        //         "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
        //         vault_address, vault_address
        //     );

        //     let collateral_link = format!(
        //         "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
        //         vault.collateral.unwrap(),
        //         vault.symbol.unwrap()
        //     );

        //     let row = row![
        //         i + 1,
        //         symbiotic_link,
        //         vault_link,
        //         collateral_link,
        //         total_stake_formatted,
        //         active_stake_formatted,
        //     ];

        //     table.add_row(row);

        //     i += 1;
        //     if i >= self.limit {
        //         break;
        //     }
        // }

        table.printstd();

        Ok(())
    }
}
