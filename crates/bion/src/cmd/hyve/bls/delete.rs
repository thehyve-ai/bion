use account_utils::OperatorDefinitions;
use clap::Parser;
use hyve_cli_runner::CliContext;

use std::fs;

use crate::common::DirsCliArgs;

#[derive(Debug, Parser)]
#[clap(about = "Delete operator and validator keys.")]
pub struct DeleteCommand {
    #[arg(
        long,
        required = true,
        value_name = "VOTING_PUBKEY",
        help = "The public key of the BLS keystore."
    )]
    voting_pubkey: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,
}

impl DeleteCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let operators_dir = self.dirs.operators_dir();
        let mut pubkey = self.voting_pubkey;
        if !pubkey.starts_with("0x") {
            pubkey = format!("0x{}", pubkey);
        }

        let mut operator_defs = OperatorDefinitions::open(&operators_dir)
            .map_err(|e| eyre::eyre!(format!("Unable to open {:?}: {:?}", &operators_dir, e)))?;

        if !operator_defs.remove(&pubkey) {
            return Err(eyre::eyre!("Operator not found"));
        }

        operator_defs
            .save(&operators_dir)
            .map_err(|e| eyre::eyre!(format!("Unable to save {:?}: {:?}", &operators_dir, e)))?;

        let dest_dir = &operators_dir.join(pubkey);

        fs::remove_dir_all(dest_dir)?;

        Ok(())
    }
}
