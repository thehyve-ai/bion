use alloy_primitives::Address;
use cast::Cast;
use clap::Parser;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use std::str::FromStr;

use crate::{
    common::{
        consts::{TESTNET_ADDRESSES, TESTNET_VAULTS},
        DirsCliArgs,
    },
    utils::load_from_json_file,
};

use super::{
    is_operator, is_opted_in, ImportedAddresses, IMPORTED_ADDRESSES_DIR, IMPORTED_ADDRESSES_FILE,
};

const VAULT_OPT_IN_ENTITY: &str = "vault_opt_in_service";
const NETWORK_OPT_IN_ENTITY: &str = "network_opt_in_service";
const HYVE_NETWORK_ENTITY: &str = "hyve_network";

#[derive(Debug, Parser)]
#[clap(about = "List tracked operators with metadata.")]
pub struct ListCommand {
    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl ListCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let operators_dir = self.dirs.operators_dir();
        let imported_addresses_dir = operators_dir.join(IMPORTED_ADDRESSES_DIR);
        let imported_addresses_file = imported_addresses_dir.join(IMPORTED_ADDRESSES_FILE);

        let imported_addresses = match load_from_json_file(imported_addresses_file.clone()) {
            Ok(imported_addresses) => imported_addresses,
            Err(..) => ImportedAddresses::new(),
        };

        let config = self.eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let eth_client = Cast::new(provider);

        let mut table = Table::new();

        // table headers
        table.add_row(row![
            "alias",
            "address",
            "is_operator",
            "registered_vaults",
            "is_hyve_operator"
        ]);
        for (address, alias) in imported_addresses.iter() {
            let is_operator = is_operator(address.clone(), &eth_client).await?;
            let mut registered_vaults = vec![];
            for (token, vault_address) in TESTNET_VAULTS.entries() {
                let is_opted_in = is_opted_in(
                    address.clone(),
                    Address::from_str(vault_address)?,
                    Address::from_str(TESTNET_ADDRESSES[VAULT_OPT_IN_ENTITY])?,
                    &eth_client,
                )
                .await?;

                if is_opted_in {
                    registered_vaults.push(String::from_str(token)?);
                }
            }

            if registered_vaults.is_empty() {
                registered_vaults.push("None".to_string());
            }

            let is_hyve_operator = is_opted_in(
                address.clone(),
                Address::from_str(TESTNET_ADDRESSES[HYVE_NETWORK_ENTITY])?,
                Address::from_str(TESTNET_ADDRESSES[NETWORK_OPT_IN_ENTITY])?,
                &eth_client,
            )
            .await?;

            table.add_row(row![
                alias.clone().unwrap_or_else(|| "None".to_string()),
                address,
                is_operator,
                registered_vaults.join(", "),
                is_hyve_operator
            ]);
        }

        table.printstd();

        Ok(())
    }
}
