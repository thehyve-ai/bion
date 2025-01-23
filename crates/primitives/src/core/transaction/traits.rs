use std::fmt;

use alloy_primitives::{keccak256, B256};
use alloy_rlp::BufMut;

use crate::aliases::DacId;

use super::signed::Signed;

/// Represents a minimal Hyve transaction.
///
/// Note: inspired by Reth
pub trait Transaction: fmt::Debug + Send + Sync + 'static {
    fn dac_id(&self) -> Option<DacId>;

    /// Get `nonce`.
    fn nonce(&self) -> u64;

    /// Returns the transaction type
    fn ty(&self) -> u8;

    /// Blob versioned hashes for eip4844 transaction. For previous transaction types this is
    /// `None`.
    fn blob_versioned_hashes(&self) -> Option<&[B256]>;
}

/// From Reth.
///
/// A signable transaction.
///
/// A transaction can have multiple signature types. This is usually
/// [`alloy_primitives::Signature`], however, it may be different for future EIP-2718 transaction
/// types, or in other networks. For example, in Optimism, the deposit transaction signature is the
/// unit type `()`.
pub trait SignableTransaction<Signature>: Transaction {
    /// Sets `chain_id`.
    ///
    /// Prefer [`set_chain_id_checked`](Self::set_chain_id_checked).
    fn set_dac_id(&mut self, dac_id: DacId);

    /// Set `dac_id` if it is not already set. Checks that the provided `dac_id` matches the
    /// existing `dac_id` if it is already set, returning `false` if they do not match.
    fn set_dac_id_checked(&mut self, dac_id: DacId) -> bool {
        match self.dac_id() {
            Some(tx_dac_id) => {
                if tx_dac_id != dac_id {
                    return false;
                }
                self.set_dac_id(dac_id);
            }
            None => {
                self.set_dac_id(dac_id);
            }
        }
        true
    }

    /// RLP-encodes the transaction for signing.
    fn encode_for_signing(&self, out: &mut dyn BufMut);

    /// Outputs the length of the signature RLP encoding for the transaction.
    fn payload_len_for_signature(&self) -> usize;

    /// RLP-encodes the transaction for signing it. Used to calculate `signature_hash`.
    ///
    /// See [`SignableTransaction::encode_for_signing`].
    fn encoded_for_signing(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.payload_len_for_signature());
        self.encode_for_signing(&mut buf);
        buf
    }

    /// Calculate the signing hash for the transaction.
    fn signature_hash(&self) -> B256 {
        keccak256(self.encoded_for_signing())
    }

    /// Convert to a signed transaction by adding a signature and computing the
    /// hash.
    fn into_signed(self, signature: Signature) -> Signed<Self, Signature>
    where
        Self: Sized;
}
