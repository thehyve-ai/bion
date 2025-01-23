use async_trait::async_trait;
use clap::Parser;
use hyve_cli_runner::CliContext;
use hyve_primitives::alloy_primitives::Address;

use crate::{
    common::DirsCliArgs,
    utils::{load_from_json_file, write_to_json_file},
};

use super::{ImportedAddresses, IMPORTED_ADDRESSES_DIR, IMPORTED_ADDRESSES_FILE};

#[derive(Debug, Parser)]
#[clap(about = "Stop tracking the state of a local operator.")]
pub struct DeleteCommand {
    #[arg(
        long,
        required = true,
        value_name = "KEY",
        help = "Either the address or alias to delete."
    )]
    key: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,
}

impl DeleteCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let operators_dir = self.dirs.operators_dir();
        let imported_addresses_dir = operators_dir.join(IMPORTED_ADDRESSES_DIR);
        let imported_addresses_file = imported_addresses_dir.join(IMPORTED_ADDRESSES_FILE);

        let mut imported_addresses = match load_from_json_file(imported_addresses_file.clone()) {
            Ok(imported_addresses) => imported_addresses,
            Err(..) => ImportedAddresses::new(),
        };

        if imported_addresses.is_empty() {
            return Err(eyre::eyre!("No addresses or aliases found to delete"));
        }

        if let Ok(address) = &self.key.parse::<Address>() {
            if imported_addresses.remove(address).is_none() {
                return Err(eyre::eyre!("Address or alias was not found"));
            }
        } else {
            let mut key_to_remove = None;
            for (address, alias) in imported_addresses.iter() {
                match alias {
                    Some(alias) => {
                        if alias == &self.key {
                            key_to_remove = Some(address.clone());
                            break;
                        }
                    }
                    None => continue,
                }
            }

            if key_to_remove.is_some() {
                imported_addresses.remove(&key_to_remove.unwrap());
            } else {
                return Err(eyre::eyre!("Address or alias was not found"));
            }
        }

        write_to_json_file(imported_addresses_file, &imported_addresses, false)
            .map_err(|e| eyre::eyre!(e))?;

        Ok(())
    }
}
