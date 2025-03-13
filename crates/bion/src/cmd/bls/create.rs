use account_utils::{
    helpers::{random_password_string, strip_off_newlines},
    mnemonic::{mnemonic_from_phrase, random_mnemonic},
    OperatorDefinition, OperatorDefinitions, PasswordStorage, ZeroizeString, CONFIG_FILENAME,
};
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use eth2_wallet::WalletBuilder;
use hyve_cli_runner::CliContext;
use tracing::debug;

use std::io::Write;
use std::{fs, path::PathBuf, str::from_utf8};

use crate::{
    common::{consts::DEFAULT_KEYSTORE_FILENAME, DirsCliArgs},
    utils::write_to_json_file,
};

#[derive(Debug, Parser)]
#[clap(about = "Create operators and validator keys.")]
pub struct CreateCommand {
    #[arg(
        long,
        value_name = "MNEMONIC",
        help = "The mnemonic to use for the operator. The mnemonic must be 24 words long and all words should be space seperated.",
        conflicts_with = "mnemonic_file"
    )]
    mnemonic: Option<String>,

    #[arg(
        long,
        value_name = "MNEMONIC_FILE",
        help = "The file containing the mnemonic to use for the operator.",
        conflicts_with = "mnemonic"
    )]
    mnemonic_file: Option<PathBuf>,

    #[arg(long, value_name = "CHAIN_ID", help = "The chain ID of the network.")]
    chain_id: Option<u64>,

    #[clap(flatten)]
    dirs: DirsCliArgs,
}

#[derive(Debug, thiserror::Error)]
pub enum ExecuteError {
    #[error("User cancelled")]
    UserCancelled,

    #[error("Ignorable error")]
    Ignore,

    #[error("Other error: {0}")]
    Other(#[from] eyre::Error),
}

type Result<T> = std::result::Result<T, ExecuteError>;

impl CreateCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        match self.execute_inner().await {
            Ok(_) => Ok(()),
            Err(e) => {
                debug!(target: "hyve::cli::operator::bls::create", "Failed to create operator: {}", e.to_string());

                match e {
                    ExecuteError::UserCancelled => {
                        println!("{}", "User cancelled due to ctrl-c".bright_cyan());
                    }
                    ExecuteError::Ignore => {
                        println!("{}", "User cancelled due to ignore".bright_cyan());
                    }
                    _ => {
                        println!("{}", "Failed to create operator due to error".bright_red());
                        println!("{}", e.to_string().bright_red());
                    }
                }
                Ok(())
            }
        }
    }
}

impl CreateCommand {
    async fn execute_inner(self) -> Result<()> {
        println!(
            "\n{}",
            "🔑 Starting BLS key creation...".bold().bright_cyan()
        );
        println!("\n");

        // Step 1: Mnemonic Setup
        println!("{}", "Step 1/3: Mnemonic Setup".bold().bright_cyan());

        let mnemonic = self.handle_mnemonic_setup().await?;
        println!();
        println!();

        // Step 2: Keystore Password Setup
        println!(
            "{}",
            "Step 2/3: Keystore Password Setup".bold().bright_cyan()
        );
        println!("{}", "Starting keystore creation...".yellow());
        let (wallet, keystore_password, wallet_password) = self.setup_keystore(&mnemonic).await?;
        println!("✅ {}", "Keystore password setup complete".bright_green());
        println!();
        println!();

        // Step 3: BLS Key Creation
        println!("{}", "Step 3/3: BLS Key Creation".bold().bright_cyan());
        let (voting_pubkey, keystore_path, definitions_file) =
            self.create_bls_keys(wallet, keystore_password, wallet_password.as_ref())?;
        println!();
        println!();
        println!();
        println!();

        // Summary
        println!("✨ {}", "New operator created".bright_green().bold(),);

        println!();
        println!("{}", "Summary".bold().bright_cyan());
        println!("{}", "─".repeat(20));
        println!("{}", "BLS public key:".bright_white().bold());
        println!("{}", voting_pubkey.to_string().bright_yellow());
        println!("{}", "Operator keystore:".bright_white().bold());
        println!("{}", keystore_path.bright_yellow());
        println!("{}", "Defintions file:".bright_white().bold());
        println!("{}", definitions_file.to_string_lossy().bright_yellow());
        println!("{}", "─".repeat(20));
        Ok(())
    }
}

impl CreateCommand {
    pub fn new(
        mnemonic: Option<String>,
        mnemonic_file: Option<PathBuf>,
        dirs: DirsCliArgs,
        chain_id: Option<u64>,
    ) -> Self {
        Self {
            mnemonic,
            mnemonic_file,
            dirs,
            chain_id,
        }
    }

    pub fn create_bls_keys(
        &self,
        mut wallet: eth2_wallet::Wallet,
        keystore_password: Option<ZeroizeString>,
        wallet_password: &[u8],
    ) -> Result<(String, String, PathBuf)> {
        println!(
            "{}",
            "🔄 Deriving BLS keys from keystore. This may take up to 2 minutes...".bright_cyan()
        );

        let operators_dir = self.dirs.operators_dir(self.chain_id)?;
        let mut defs = OperatorDefinitions::open_or_create(&operators_dir)
            .map_err(|e| eyre::eyre!(format!("Unable to open {:?}: {:?}", &operators_dir, e)))?;

        let keystore_password = keystore_password
            .clone()
            .unwrap_or_else(random_password_string);

        wallet.set_nextaccount(0).map_err(|e| {
            eyre::eyre!(format!(
                "Failure to set validator derivation index: {:?}",
                e
            ))
        })?;

        let keystores = wallet
            .next_validator(wallet_password, keystore_password.as_ref(), wallet_password)
            .map_err(|e| eyre::eyre!(format!("Failed to derive keystore: {:?}", e)))?;

        let voting_keystore = keystores.voting;
        let voting_pubkey = voting_keystore.pubkey();

        let dest_dir = &operators_dir.join(format!("0x{}", voting_pubkey));
        fs::create_dir_all(&dest_dir)
            .map_err(|e| eyre::eyre!(format!("Unable to create import directory: {:?}", e)))?;

        let dest_keystore = dest_dir.join(DEFAULT_KEYSTORE_FILENAME);
        let keystore_path = dest_keystore.to_string_lossy().to_string();

        write_to_json_file(dest_keystore.clone(), &voting_keystore, true)
            .map_err(|e| eyre::eyre!(e))?;

        let def = OperatorDefinition::new_keystore_with_password(
            dest_keystore,
            PasswordStorage::ValidatorDefinitions(keystore_password),
        )
        .map_err(|e| {
            eyre::eyre!(format!(
                "Unable to create new validator definition: {:?}",
                e
            ))
        })?;

        defs.push(def);

        defs.save(&operators_dir)
            .map_err(|e| eyre::eyre!(format!("Unable to save {:?}: {:?}", &operators_dir, e)))?;

        println!("{}", "✅ BLS key creation complete".bright_green());

        let operator_definitions_file = operators_dir.join(CONFIG_FILENAME);
        Ok((
            voting_pubkey.to_string(),
            keystore_path,
            operator_definitions_file,
        ))
    }

    pub async fn setup_keystore(
        &self,
        mnemonic: &account_utils::bip39::Mnemonic,
    ) -> Result<(eth2_wallet::Wallet, Option<ZeroizeString>, ZeroizeString)> {
        let wallet_password: ZeroizeString = random_password_string();
        let wallet = self
            .create_wallet(mnemonic, wallet_password.clone())
            .await?;

        let keystore_password = self.get_keystore_password()?;
        Ok((wallet, keystore_password, wallet_password))
    }

    async fn create_wallet(
        &self,
        mnemonic: &account_utils::bip39::Mnemonic,
        wallet_password: ZeroizeString,
    ) -> Result<eth2_wallet::Wallet> {
        let mnemonic = mnemonic.clone();
        let wallet: eth2_wallet::Wallet =
            tokio::task::spawn_blocking(move || -> Result<eth2_wallet::Wallet> {
                Ok(WalletBuilder::from_mnemonic(
                    &eth2_wallet::bip39::Mnemonic::from_phrase(
                        mnemonic.phrase(),
                        eth2_wallet::bip39::Language::English,
                    )
                    .map_err(|e| {
                        eyre::eyre!(format!("Unable to create mnemonic from phrase: {:?}", e))
                    })?,
                    wallet_password.as_ref(),
                    "".to_string(),
                )
                .map_err(|e| eyre::eyre!(format!("Unable create seed from mnemonic: {:?}", e)))?
                .build()
                .map_err(|e| eyre::eyre!(format!("Unable to create wallet: {:?}", e)))?)
            })
            .await
            .unwrap()?;
        Ok(wallet)
    }

    pub async fn handle_mnemonic_setup(&self) -> Result<account_utils::bip39::Mnemonic> {
        match (self.mnemonic.clone(), self.mnemonic_file.clone()) {
            (Some(mnemonic), None) => self.handle_cli_mnemonic(mnemonic).await,
            (None, Some(mnemonic_file)) => self.handle_file_mnemonic(mnemonic_file).await,
            (None, None) => self.handle_random_mnemonic().await,
            _ => Err(eyre::eyre!(
                "Please provide either a mnemonic phrase or a mnemonic file, but not both"
            )
            .into()),
        }
    }

    async fn handle_cli_mnemonic(
        &self,
        mnemonic: String,
    ) -> Result<account_utils::bip39::Mnemonic> {
        println!(
            "{}",
            "→ Using mnemonic phrase from command line argument".bright_cyan()
        );

        let mnemonic = mnemonic_from_phrase(mnemonic.as_str()).map_err(|e| eyre::eyre!(e))?;
        confirm_mnemonic(&mnemonic).await?;

        println!("{}", "✅ Mnemonic confirmed.".bright_green());
        Ok(mnemonic)
    }

    async fn handle_file_mnemonic(
        &self,
        mnemonic_file: PathBuf,
    ) -> Result<account_utils::bip39::Mnemonic> {
        let file_path = mnemonic_file.to_string_lossy().to_string();
        println!(
            "{} {}",
            "→ Using mnemonic phrase from file: ".bright_cyan(),
            file_path.bright_cyan()
        );

        let bytes = fs::read(mnemonic_file)
            .map_err(|e| eyre::eyre!(format!("Failed to read mnemonic file. {:?}", e)))?;
        let stripped = strip_off_newlines(bytes);
        let phrase = from_utf8(stripped.as_slice()).map_err(|e| {
            eyre::eyre!(format!(
                "Failed to convert mnemonic file to string. {:?}",
                e
            ))
        })?;
        let mnemonic = mnemonic_from_phrase(phrase).map_err(|e| eyre::eyre!(e))?;

        confirm_mnemonic(&mnemonic).await?;
        println!("{}", "✅ Mnemonic confirmed. ".bright_green());
        Ok(mnemonic)
    }

    async fn handle_random_mnemonic(&self) -> Result<account_utils::bip39::Mnemonic> {
        println!(
            "{}",
            "No mnemonic provided. Would you like to generate a random mnemonic? [y/N]"
                .bright_cyan()
        );

        loop {
            let input: String = read_user_confirmation()?;

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => return self.generate_and_confirm_mnemonic().await,
                "" | "n" | "no" => {
                    println!("\n{}", "Mnemonic generation cancelled".bright_cyan());
                    return Err(ExecuteError::Ignore);
                }
                _ => println!("{}", "Please enter 'y/yes' or 'n/no'".bright_cyan()),
            }
        }
    }

    async fn generate_and_confirm_mnemonic(&self) -> Result<account_utils::bip39::Mnemonic> {
        let mnemonic = random_mnemonic();
        println!();
        println!(
            "{}",
            "⚠ WARNING: Please store this mnemonic phrase safely. It will not be shown again!"
                .bright_yellow()
                .bold()
        );
        println!("\nYour mnemonic phrase:");
        println!("{}", "─".repeat(20));
        println!("{}", mnemonic.phrase().bright_white().bold());
        println!("{}", "─".repeat(20));
        println!(
            "{}",
            "Please type 'yes' after you have safely stored this mnemonic phrase:".bright_cyan()
        );

        loop {
            let confirmation: String = read_user_confirmation()?;
            if confirmation.trim() == "yes" {
                clear_previous_lines(10);
                println!(
                    "{}",
                    "✅ Mnemonic phrase confirmed as backed up.".bright_green()
                );
                return Ok(mnemonic);
            } else {
                clear_previous_lines(10);
                println!("{}", "Mnemonic confirmation cancelled".bright_cyan());
                return Err(ExecuteError::Ignore);
            }
        }
    }
}

impl CreateCommand {
    fn get_keystore_password(&self) -> Result<Option<ZeroizeString>> {
        let options = vec![
            "Enter a custom password",
            "Continue without a password (not recommended)",
            "Generate a random strong password",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("\nChoose a password option for the BLS keystore")
            .items(&options)
            .default(0)
            .interact()
            .map_err(|e| eyre::eyre!(format!("Failed to show password selection menu: {}", e)))?;

        match selection {
            0 => {
                println!("{}", "Please enter a password when prompted.".bright_cyan());
                Ok(Some(self.get_keystore_password_from_input()?))
            }
            1 => Ok(None),
            2 => Ok(Some(self.get_random_keystore_password()?)),
            _ => unreachable!(),
        }
    }

    fn get_keystore_password_from_input(&self) -> Result<ZeroizeString> {
        let password = loop {
            let password = rpassword::prompt_password_stderr("Enter your password:")
                .map_err(|e| format!("Error reading from stdin: {}", e))
                .map(ZeroizeString::from)
                .map_err(|e| eyre::eyre!(e))?;

            let confirmation = rpassword::prompt_password_stderr("Confirm your password:")
                .map_err(|e| eyre::eyre!("Error reading from stdin: {}", e))?;

            if password.as_str() != confirmation {
                clear_previous_lines(2);
                println!(
                    "\n{}",
                    "❌ Passwords do not match. Please try again.".bright_red()
                );
                continue;
            }
            if password.as_str().trim().is_empty() {
                clear_previous_lines(2);
                println!(
                    "\n{}",
                    "❌ Password cannot be empty. Please try again.".bright_red()
                );

                continue;
            }
            clear_previous_lines(3);
            break password;
        };

        Ok(password)
    }

    fn get_random_keystore_password(&self) -> Result<ZeroizeString> {
        let password = random_password_string();

        println!(
            "\n⚠️  {}",
            "WARNING: Please store this password safely. It will not be shown again!"
                .bright_yellow()
                .bold()
        );
        println!("{}", "Your password is:".bright_white());
        println!("{}", "─".repeat(20));
        println!("{}", password.as_str().bright_yellow().bold());
        println!("{}", "─".repeat(20));
        println!(
            "{}",
            "Please type 'yes' after you have safely stored this password:".bright_cyan()
        );

        loop {
            let confirmation: String = read_user_confirmation()?;

            if confirmation.trim() == "yes" {
                println!("\n{}", "✅ Password confirmed as backed up.".bright_green());
                break;
            } else {
                println!("{}", "Password confirmation cancelled".bright_cyan());
                return Err(ExecuteError::Ignore);
            }
        }
        Ok(password)
    }
}

async fn confirm_mnemonic(mnemonic: &account_utils::bip39::Mnemonic) -> Result<()> {
    println!(
        "{}",
        "Please confirm this is the mnemonic you want to use:"
            .bright_white()
            .bold()
    );
    println!("\nYour mnemonic phrase:");
    println!("{}", "─".repeat(20));
    println!("{}", mnemonic.phrase().bright_white().bold());
    println!("{}", "─".repeat(20));
    println!(
        "{}",
        "Please confirm you want to use this mnemonic [y/n]:".bright_cyan()
    );

    loop {
        let confirmation: String = read_user_confirmation()?;
        match confirmation.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                break Ok(());
            }
            "n" | "no" => {
                println!("{}", "Mnemonic confirmation cancelled".bright_cyan());
                return Err(ExecuteError::Ignore);
            }
            _ => {
                println!(
                    "{}",
                    "Please type 'y/yes' to confirm or 'n/no' to cancel:".bright_cyan()
                );
            }
        }
    }
}

fn clear_previous_lines(num_lines: u16) {
    print!("\x1B[{}F", num_lines); // Move up N lines
    print!("\x1B[J"); // Clear from cursor to end of screen
    print!("\x1B[0G"); // Move to start of line
    std::io::stdout().flush().unwrap();
}

fn read_user_confirmation() -> Result<String> {
    Ok(Input::with_theme(&ColorfulTheme::default())
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            let normalized = input.trim().to_lowercase();
            match normalized.as_str() {
                "y" | "yes" | "n" | "no" => Ok(()),
                _ => Err("Please type 'y/yes' or 'n/no'"),
            }
        })
        .interact()
        .map_err(|e: dialoguer::Error| match e {
            dialoguer::Error::IO(e) => match e.kind() {
                std::io::ErrorKind::Interrupted => ExecuteError::UserCancelled,
                _ => ExecuteError::Other(e.into()),
            },
        })?)
}
