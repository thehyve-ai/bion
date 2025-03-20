use std::{
    future::Future,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use alloy_primitives::{Address, U256};
use dialoguer::{theme::ColorfulTheme, Input};

use crate::{transaction_data::SafeTransactionData, SafeMetaTransaction};

#[derive(Debug, thiserror::Error)]
pub enum ExecuteError {
    #[error("User cancelled")]
    UserCancelled,

    #[allow(dead_code)]
    #[error("Ignorable error")]
    Ignore,

    #[error("Other error: {0}")]
    Other(#[from] eyre::Error),
}

pub fn build_safe_tx(tx: SafeMetaTransaction, nonce: U256) -> eyre::Result<SafeTransactionData> {
    Ok(SafeTransactionData {
        to: tx.to.to_checksum(None),
        value: tx.value.try_into()?,
        data: tx.input,
        operation: 0,
        safe_tx_gas: 0,
        base_gas: 0,
        gas_price: 0,
        gas_token: Address::ZERO,
        refund_receiver: Address::ZERO,
        nonce: nonce.try_into()?,
    })
}

/// Async version that runs a loading animation until a future completes
pub async fn print_loading_until_async<F, T>(message: &str, future: F) -> T
where
    F: Future<Output = T>,
{
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let message = message.to_string(); // Clone the message string

    // Spawn the animation in a separate thread
    std::thread::spawn(move || {
        let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let mut i = 0;

        while running_clone.load(Ordering::Relaxed) {
            print!("\r{} {} ", message, spinner[i]);
            std::io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(100));
            i = (i + 1) % spinner.len();
        }

        print!("\r{}\r", " ".repeat(message.len() + 2));
        std::io::stdout().flush().unwrap();
    });

    // Wait for the future to complete
    let result = future.await;

    // Stop the animation
    running.store(false, Ordering::Relaxed);
    std::thread::sleep(std::time::Duration::from_millis(100)); // Give animation thread time to clean up

    result
}

pub fn read_user_confirmation() -> eyre::Result<String> {
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

// /// Adjusts the V value in a signature for Safe compatibility
// ///
// /// This handles different signing methods and ensures the V value is correct for Safe transactions
// pub async fn adjust_v_in_signature(
//     signature: PrimitiveSignature,
//     tx_hash: B256,
//     signer_address: Address,
// ) -> eyre::Result<Vec<u8>> {
//     const ETHEREUM_V_VALUES: [u8; 4] = [0, 1, 27, 28];
//     const MIN_VALID_V_VALUE_FOR_SAFE_ECDSA: u8 = 27;

//     let is_prefixed = is_tx_hash_signed_with_prefix(&tx_hash, signature, signer_address).await?;

//     let signature_hex = signature.as_bytes().encode_hex_with_prefix();
//     let signature_v = u8::from_str_radix(&signature_hex[signature_hex.len() - 2..], 16)?;

//     /*
//       The Safe's expected V value for ECDSA signature is:
//       - 27 or 28
//       - 31 or 32 if the message was signed with a EIP-191 prefix. Should be calculated as ECDSA V value + 4
//       Some wallets do that, some wallets don't, V > 30 is used by contracts to differentiate between
//       prefixed and non-prefixed messages. The only way to know if the message was signed with a
//       prefix is to check if the signer address is the same as the recovered address.

//       More info:
//       https://docs.safe.global/safe-core-protocol/signatures
//     */
//     if signature_v < MIN_VALID_V_VALUE_FOR_SAFE_ECDSA {
//         signature_v += MIN_VALID_V_VALUE_FOR_SAFE_ECDSA
//     }

//     let adjust_signature =
//         format!("{}{}", &signature_hex[..signature_hex.len() - 2], format!("{:02x}", signature_v));

//     let signature_hex = signature.encode_hex_with_prefix();
//     let signature_v = u8::from_str_radix(&signature_hex[signature_hex.len() - 2..], 16)?;
//     if !ETHEREUM_V_VALUES.contains(&signature_v) {
//         return Err(eyre::eyre!("Invalid signature V value"));
//     }

//     /*
//       The Safe's expected V value for ECDSA signature is:
//       - 27 or 28
//       - 31 or 32 if the message was signed with a EIP-191 prefix. Should be calculated as ECDSA V value + 4
//       Some wallets do that, some wallets don't, V > 30 is used by contracts to differentiate between
//       prefixed and non-prefixed messages. The only way to know if the message was signed with a
//       prefix is to check if the signer address is the same as the recovered address.

//       More info:
//       https://docs.safe.global/safe-core-protocol/signatures
//     */
//     if signature_v < MIN_VALID_V_VALUE_FOR_SAFE_ECDSA {
//         signature_v += MIN_VALID_V_VALUE_FOR_SAFE_ECDSA
//     }

//     let mut r = [0u8; 32];
//     let mut s = [0u8; 32];
//     r.copy_from_slice(&signature[0..32]);
//     s.copy_from_slice(&signature[32..64]);
//     let mut v = signature[64];

//     if !ETHEREUM_V_VALUES.contains(&v) {
//         return Err(eyre::eyre!("Invalid signature V value"));
//     }

//     // For ETH_SIGN (sign_message), adjust V value
//     if v < MIN_VALID_V_VALUE_FOR_SAFE_ECDSA {
//         v += MIN_VALID_V_VALUE_FOR_SAFE_ECDSA;
//     }

//     // Check if the message was signed with a prefix
//     let sig =
//         Signature { r: FixedBytes::from_slice(&r), s: FixedBytes::from_slice(&s), v: v as u64 };

//     // Reconstruct the signature with the adjusted v value
//     let mut adjusted_signature = Vec::with_capacity(65);
//     adjusted_signature.extend_from_slice(&r);
//     adjusted_signature.extend_from_slice(&s);

//     // Check if the signature was created with a prefix
//     let recovered_address = recover_signer(tx_hash, &sig)?;
//     if recovered_address != signer_address {
//         // If addresses don't match, the message was likely signed with a prefix
//         // Add 4 to V as per Safe specification
//         v += 4;
//     }

//     adjusted_signature.push(v);
//     Ok(adjusted_signature)
// }

// /// Checks if a transaction hash was signed with an EIP-191 prefix
// ///
// /// Returns true if the signature was created with a prefix, false otherwise
// pub async fn is_tx_hash_signed_with_prefix(
//     tx_hash: &B256,
//     signature: PrimitiveSignature,
//     owner_address: Address,
// ) -> eyre::Result<bool> {
//     match signature.recover_address_from_msg(tx_hash) {
//         Ok(address) => {
//             if address != owner_address {
//                 Ok(true)
//             } else {
//                 Ok(false)
//             }
//         }
//         Err(_) => Ok(true),
//     }
// }
