use async_trait::async_trait;
use clap::Parser;
use hyve_cli_runner::CliContext;
use hyve_primitives::alloy_primitives::Address;

use std::fs::create_dir_all;

use crate::{
    common::DirsCliArgs,
    utils::{load_from_json_file, write_to_json_file},
};

use super::{ImportedAddresses, IMPORTED_ADDRESSES_DIR, IMPORTED_ADDRESSES_FILE};

#[derive(Debug, Parser)]
#[clap(about = "Import an address and store it locally.")]
pub struct ImportCommand {
    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "The address to import."
    )]
    address: Address,

    #[arg(
        long,
        value_name = "ALIAS",
        help = "The alias to be used for the address."
    )]
    alias: Option<String>,

    #[clap(flatten)]
    dirs: DirsCliArgs,
}

impl ImportCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let operators_dir = self.dirs.operators_dir();
        let imported_addresses_dir = operators_dir.join(IMPORTED_ADDRESSES_DIR);
        let imported_addresses_file = imported_addresses_dir.join(IMPORTED_ADDRESSES_FILE);

        let mut create_new = false;
        let mut imported_addresses = match load_from_json_file(imported_addresses_file.clone()) {
            Ok(imported_addresses) => imported_addresses,
            Err(..) => {
                create_dir_all(&imported_addresses_dir).map_err(|e| {
                    eyre::eyre!(format!(
                        "Unable to create import directory: {:?}: {:?}",
                        imported_addresses_dir, e
                    ))
                })?;

                create_new = true;
                ImportedAddresses::new()
            }
        };

        imported_addresses
            .entry(self.address)
            .or_insert_with(|| self.alias);

        write_to_json_file(imported_addresses_file, &imported_addresses, create_new)
            .map_err(|e| eyre::eyre!(e))?;

        Ok(())
    }
}
