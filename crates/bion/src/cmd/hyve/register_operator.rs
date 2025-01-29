use account_utils::OperatorDefinitions;
use alloy_primitives::{
    hex::{FromHex, ToHexExt},
    keccak256, Address, Bytes,
};
use alloy_rlp::RlpEncodable;
use clap::Parser;
use ethereum_types::H256;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_wallets::WalletSigner;
use hyve_cli_runner::CliContext;
use libp2p::{
    identity::{secp256k1, Keypair},
    PeerId,
};
use tracing::trace;

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::utils::get_chain_id,
    common::DirsCliArgs,
    hyve::consts::get_hyve_middleware_service,
    symbiotic::{calls::is_vault, consts::get_vault_factory},
    utils::validate_cli_args,
};

const HYVE_MIDDLEWARE_ENTITY: &str = "hyve_middleware_service";

#[derive(RlpEncodable)]
pub struct Keys {
    pub bls: Bytes,
    pub p2p: Bytes,
}

#[derive(Debug, Parser)]
#[clap(about = "Register an Operator with a BLS key in the HyveDA middleware.")]
pub struct RegisterOperatorCommand {
    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "Address of the operator."
    )]
    address: Address,

    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "The address of the vault to opt-in."
    )]
    vault_address: Address,

    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "The pubkey of the BLS keystore."
    )]
    voting_pubkey: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    tx: TransactionOpts,

    #[clap(flatten)]
    eth: EthereumOpts,

    /// Timeout for sending the transaction.
    #[arg(long, env = "ETH_TIMEOUT")]
    pub timeout: Option<u64>,

    /// The number of confirmations until the receipt is fetched.
    #[arg(long, default_value = "1")]
    confirmations: u64,
}

impl RegisterOperatorCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            vault_address,
            voting_pubkey,
            dirs,
            tx,
            eth,
            timeout,
            confirmations,
        } = self;

        validate_cli_args(Some(address), &eth).await?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        let middleware_service = get_hyve_middleware_service(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;

        let is_vault = is_vault(vault_address, vault_factory, &provider).await?;
        if !is_vault {
            return Err(eyre::eyre!("Address is not a vault."));
        }

        let operators_dir = dirs.operators_dir();
        let mut pubkey = voting_pubkey;
        if !pubkey.starts_with("0x") {
            pubkey = format!("0x{}", pubkey);
        }

        let operators = OperatorDefinitions::open(operators_dir)
            .map_err(|e| eyre::eyre!("Error while loading operator definitions: {:?}", e))?;

        let bls_keypair = operators
            .as_slice()
            .iter()
            .find(|op| op.public_key.to_string().to_lowercase() == pubkey.to_lowercase())
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

                trace!("Retrieved keystore password, trying to decrypt keystore");

                keystore_password
                    .map(|password| {
                        voting_keystore
                            .decrypt_keypair(password.as_str().as_bytes())
                            .map_err(|e| eyre::eyre!("Error decrypting keystore: {:?}", e))
                            .map(|keypair| {
                                trace!("Keystore decrypted successfully");
                                keypair
                            })
                    })
                    .transpose()
                    .ok()?
            })
            .ok_or_else(|| eyre::eyre!("No keypair found for the given public key"))?;

        // Retrieve the signer, and bail if it can't be constructed.
        let signer = eth.wallet.signer().await?;

        let peer_id = match &signer {
            WalletSigner::Local(s) => {
                let mut key_bytes = s.to_bytes();
                let secret_key = secp256k1::SecretKey::try_from_bytes(&mut key_bytes)?;
                let kp: secp256k1::Keypair = secret_key.into();
                let kp: Keypair = kp.into();
                PeerId::from(kp.public())
            }
            _ => {
                return Err(eyre::eyre!("Only local private key signers are supported."));
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

        let voting_pubkey = format!("0x00{}", &pubkey[2..]);

        let to = foundry_common::ens::NameOrAddress::Address(middleware_service);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some(
                "registerOperator(bytes memory key, address vault, bytes memory signature)"
                    .to_string(),
            ),
            args: vec![
                voting_pubkey,
                "0x0000000000000000000000000000000000000000".to_string(), // placeholder for now
                bls_sig,
            ],
            cast_async: true,
            confirmations,
            command: None,
            unlocked: true,
            timeout,
            tx,
            eth,
            path: None,
        };
        arg.run().await
    }
}
