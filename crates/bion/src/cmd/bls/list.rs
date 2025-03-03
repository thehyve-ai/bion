use account_utils::OperatorDefinitions;
use clap::Parser;
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use crate::common::DirsCliArgs;

#[derive(Debug, Parser)]
#[clap(about = "List local operators and validator keys.")]
pub struct ListCommand {
    #[arg(long, value_name = "CHAIN_ID", help = "The chain ID of the network.")]
    chain_id: Option<u64>,

    #[clap(flatten)]
    dirs: DirsCliArgs,
}

impl ListCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let operators_dir = self.dirs.operators_dir(self.chain_id)?;

        let operator_defs = OperatorDefinitions::open(&operators_dir)
            .map_err(|e| eyre::eyre!(format!("Unable to open {:?}: {:?}", &operators_dir, e)))?;

        let entries = operator_defs.as_slice();
        if entries.is_empty() {
            eyre::bail!("No local operators found.");
        }

        let mut table = Table::new();

        // table headers
        table.add_row(row!["enabled", "public_key", "description",]);

        for entry in entries {
            table.add_row(row![
                entry.enabled,
                entry.public_key.as_hex_string(),
                entry.description
            ]);
        }

        table.printstd();

        Ok(())
    }
}
