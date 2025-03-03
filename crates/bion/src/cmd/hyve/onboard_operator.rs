use account_utils::OperatorDefinitions;
use alloy_primitives::{
    hex::{FromHex, ToHexExt},
    keccak256, Address, Bytes,
};
use clap::Parser;
use colored::Colorize;
use ethereum_types::H256;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_common::{ens::NameOrAddress, provider::RetryProvider};
use foundry_wallets::WalletSigner;
use hyve_cli_runner::CliContext;
use libp2p::{identity::secp256k1, PeerId};
use lighthouse_bls::Keypair;

use std::path::PathBuf;

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::{
        alias_utils::{get_alias_config, set_foundry_signing_method},
        hyve::register_operator::Keys,
        utils::get_chain_id,
    },
    common::DirsCliArgs,
    hyve::consts::{get_hyve_middleware_service, get_hyve_network},
    symbiotic::{
        calls::{is_operator, is_opted_in_network},
        consts::{get_network_opt_in_service, get_operator_registry},
    },
    utils::{
        print_loading_until_async, print_success_message, read_user_confirmation, validate_cli_args,
    },
};

#[derive(Debug, Parser)]
#[clap(about = "Onboard an Operator in the HyveDA and Symbiotic.")]
pub struct OnboardOperatorCommand {
    #[arg(
        long,
        value_name = "MNEMONIC",
        help = "The mnemonic to use for the operator. The mnemonic must be 24 words long and all words should be space seperated.",
        conflicts_with = "bls_mnemonic_file"
    )]
    bls_mnemonic: Option<String>,

    #[arg(
        long,
        value_name = "MNEMONIC_FILE",
        help = "The file containing the mnemonic to use for the operator.",
        conflicts_with = "bls_mnemonic"
    )]
    bls_mnemonic_file: Option<PathBuf>,

    #[arg(
        long,
        help = "Whether or not to be prompted to specify a keystore password. Will otherwise be randomly generated."
    )]
    prompt_keystore_password: bool,

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

impl OnboardOperatorCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(mut self, _ctx: CliContext) -> eyre::Result<()> {
        validate_cli_args(&self.eth)?;

        let config = self.eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let operator_config = get_alias_config(chain_id, self.alias.clone(), &self.dirs)?;
        let operator = operator_config.address;
        let hyve_network = get_hyve_network(chain_id)?;
        let network_opt_in_service = get_network_opt_in_service(chain_id)?;
        let operator_registry = get_operator_registry(chain_id)?;
        set_foundry_signing_method(&operator_config, &mut self.eth)?;

        println!(
            "\n{}\n",
            "🚀 Starting Operator onboarding...".bold().bright_cyan()
        );

        let bls_keypair = self.create_bls_keypair(chain_id).await?;

        self.ensure_operator(operator, operator_registry, &provider)
            .await?;

        self.ensure_opted_in_network(operator, hyve_network, network_opt_in_service, &provider)
            .await?;

        self.register_key(chain_id, &bls_keypair).await?;

        println!();
        print_success_message("✅ Onboarding completed successfully");

        Ok(())
    }

    async fn create_bls_keypair(&self, chain_id: u64) -> eyre::Result<Keypair> {
        println!(
            "\n{}",
            "🔑 Starting BLS key creation...".bold().bright_cyan()
        );
        println!("\n");

        let bls_command = super::bls::create::CreateCommand::new(
            self.bls_mnemonic.clone(),
            self.bls_mnemonic_file.clone(),
            self.dirs.clone(),
        );

        // Step 1: Mnemonic Setup
        println!("{}", "Step 1/3: Mnemonic Setup".bold().bright_cyan());

        let mnemonic = bls_command.handle_mnemonic_setup().await?;
        println!();
        println!();

        // Step 2: Keystore Password Setup
        println!(
            "{}",
            "Step 2/3: Keystore Password Setup".bold().bright_cyan()
        );
        println!("{}", "Starting keystore creation...".bright_cyan());
        let (wallet, keystore_password, wallet_password) =
            bls_command.setup_keystore(&mnemonic).await?;
        println!("✅ {}", "Keystore password setup complete".bright_green());
        println!();
        println!();

        // Step 3: BLS Key Creation
        println!("{}", "Step 3/3: BLS Key Creation".bold().bright_cyan());
        let (voting_pubkey, keystore_path, definitions_file) =
            bls_command.create_bls_keys(wallet, keystore_password, wallet_password.as_ref())?;
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

        // Return the BLS keypair
        let operators_dir = self.dirs.operators_dir(Some(chain_id))?;
        let operators = OperatorDefinitions::open(&operators_dir)
            .map_err(|e| eyre::eyre!(format!("Unable to open {:?}: {:?}", &operators_dir, e)))?;

        let voting_pubkey = format!("0x{}", voting_pubkey);
        let keypair = operators
            .as_slice()
            .iter()
            .find(|op| op.public_key.to_string().to_lowercase() == voting_pubkey.to_lowercase())
            .and_then(|operator| {
                let keystore_path = operator.signing_definition.keystore_path()?;

                let voting_keystore = eth2_keystore::Keystore::from_json_file(&keystore_path)
                    .map_err(|e| eyre::eyre!(format!("Failed to read voting keystore: {:?}", e)))
                    .ok()?;
                let keystore_password = operator
                    .signing_definition
                    .keystore_password()
                    .map_err(|e| eyre::eyre!("Error loading keystore password: {:?}", e))
                    .ok()?;

                keystore_password
                    .map(|password| {
                        voting_keystore
                            .decrypt_keypair(password.as_str().as_bytes())
                            .map_err(|e| eyre::eyre!("Error decrypting keystore: {:?}", e))
                            .map(|keypair| keypair)
                    })
                    .transpose()
                    .ok()?
            })
            .ok_or_else(|| eyre::eyre!("No keypair found for the given public key"))?;

        Ok(keypair)
    }

    async fn ensure_operator(
        &self,
        operator: Address,
        operator_registry: Address,
        provider: &RetryProvider,
    ) -> eyre::Result<()> {
        println!(
            "\n{}\n",
            "📝 Ensuring Operator is registered in Symbiotic..."
                .bold()
                .bright_cyan()
        );

        let is_operator = print_loading_until_async(
            "Fetching operator status",
            is_operator(operator, operator_registry, &provider),
        )
        .await?;
        if !is_operator {
            println!(
                "{}",
                "Operator is not registered in Symbiotic.".bright_red()
            );
            println!(
                "\n {}",
                "Do you want to register in Symbiotic? (y/n)".bright_cyan()
            );

            let confirmation: String = read_user_confirmation()?;
            if confirmation.trim().to_lowercase().as_str() == "y"
                || confirmation.trim().to_lowercase().as_str() == "yes"
            {
                let to = NameOrAddress::Address(operator_registry);

                let arg = SendTxArgs {
                    to: Some(to),
                    sig: Some("registerOperator()".to_string()),
                    args: vec![],
                    cast_async: false,
                    confirmations: self.confirmations,
                    command: None,
                    unlocked: self.unlocked,
                    timeout: self.timeout,
                    tx: self.tx.clone(),
                    eth: self.eth.clone(),
                    path: None,
                };

                let _ = arg.run().await?;
                print_success_message("✅ Operator registered in Symbiotic.\n");
            } else {
                eyre::bail!("Operator must be registered in Symbiotic to continue.");
            }
        }

        Ok(())
    }

    async fn ensure_opted_in_network(
        &self,
        operator: Address,
        network: Address,
        opt_in_service: Address,
        provider: &RetryProvider,
    ) -> eyre::Result<()> {
        println!(
            "\n{}\n",
            "📝 Ensuring Operator is opted in HyveDA network..."
                .bold()
                .bright_cyan()
        );

        let is_opted_in_hyve = print_loading_until_async(
            "Fetching network opt-in status",
            is_opted_in_network(operator, network, opt_in_service, &provider),
        )
        .await?;

        if !is_opted_in_hyve {
            println!(
                "{}",
                "Operator is not opted in to the HyveDA network.".bright_red()
            );
            println!("\n {}", "Do you want to opt-in? (y/n)".bright_cyan());

            let confirmation: String = read_user_confirmation()?;
            if confirmation.trim().to_lowercase().as_str() == "y"
                || confirmation.trim().to_lowercase().as_str() == "yes"
            {
                let to = NameOrAddress::Address(opt_in_service);

                let arg = SendTxArgs {
                    to: Some(to),
                    sig: Some("optIn(address where)".to_string()),
                    args: vec![network.to_string()],
                    cast_async: false,
                    confirmations: self.confirmations,
                    command: None,
                    unlocked: self.unlocked,
                    timeout: self.timeout,
                    tx: self.tx.clone(),
                    eth: self.eth.clone(),
                    path: None,
                };

                let _ = arg.run().await?;
                print_success_message("✅ Operator opted in the HyveDA Network in Symbiotic.\n");
            } else {
                eyre::bail!("Operator must be opted in to the HyveDA network to continue.");
            }
        }

        Ok(())
    }

    async fn register_key(&self, chain_id: u64, bls_keypair: &Keypair) -> eyre::Result<()> {
        println!(
            "\n{}\n",
            "📝 Registering BLS Key in the HyveDA middleware..."
                .bold()
                .bright_cyan()
        );

        let signer = self.eth.wallet.signer().await?;

        let peer_id = match &signer {
            WalletSigner::Local(s) => {
                let mut key_bytes = s.to_bytes();
                let secret_key = secp256k1::SecretKey::try_from_bytes(&mut key_bytes)?;
                let kp: secp256k1::Keypair = secret_key.into();
                let kp: libp2p::identity::Keypair = kp.into();
                PeerId::from(kp.public())
            }
            _ => {
                eyre::bail!("Only local private key signers are supported.");
            }
        };

        let keys = Keys {
            bls: Bytes::from_hex(&bls_keypair.pk.to_string())?,
            p2p: peer_id.to_bytes().into(),
        };

        let keys_bytes = alloy_rlp::encode(keys);

        let msg = H256(keccak256(&keys_bytes.as_slice()).0);
        let bls_signed = bls_keypair.sk.sign(msg);
        let bls_sig = format!("0x{}", bls_signed.serialize().encode_hex());

        let voting_pubkey = format!("0x00{}", &bls_keypair.pk.to_string()[2..]);

        let middleware_service = get_hyve_middleware_service(chain_id)?;
        let to = foundry_common::ens::NameOrAddress::Address(middleware_service);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some(
                "registerOperator(bytes memory key, address vault, bytes memory signature)"
                    .to_string(),
            ),
            args: vec![
                voting_pubkey,
                "0x0000000000000000000000000000000000000000".to_string(), // all vaults
                bls_sig,
            ],
            cast_async: true,
            confirmations: self.confirmations,
            command: None,
            unlocked: self.unlocked,
            timeout: self.timeout,
            tx: self.tx.clone(),
            eth: self.eth.clone(),
            path: None,
        };

        let _ = arg.run().await?;

        print_success_message("✅ BLS Key registered in the HyveDA middleware");

        Ok(())
    }
}
