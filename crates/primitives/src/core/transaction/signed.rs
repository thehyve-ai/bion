use alloy_primitives::{Signature, B256};
use serde::{Deserialize, Serialize};

use super::traits::SignableTransaction;

/// A transaction with a signature and hash seal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signed<T, Sig = Signature> {
    #[serde(flatten)]
    tx: T,
    #[serde(flatten)]
    signature: Sig,

    hash: B256,
}

impl<T, Sig> Signed<T, Sig> {
    /// Returns a reference to the transaction.
    #[doc(alias = "transaction")]
    pub const fn tx(&self) -> &T {
        &self.tx
    }

    /// Returns a reference to the signature.
    pub const fn signature(&self) -> &Sig {
        &self.signature
    }

    /// Returns a reference to the transaction hash.
    #[doc(alias = "tx_hash", alias = "transaction_hash")]
    pub const fn hash(&self) -> &B256 {
        &self.hash
    }

    /// Splits the transaction into parts.
    pub fn into_parts(self) -> (T, Sig, B256) {
        (self.tx, self.signature, self.hash)
    }

    /// Returns the transaction without signature.
    pub fn strip_signature(self) -> T {
        self.tx
    }
}

impl<T: SignableTransaction<Sig>, Sig> Signed<T, Sig> {
    /// Instantiate from a transaction and signature. Does not verify the signature.
    pub const fn new_unchecked(tx: T, signature: Sig, hash: B256) -> Self {
        Self {
            tx,
            signature,
            hash,
        }
    }

    /// Calculate the signing hash for the transaction.
    pub fn signature_hash(&self) -> B256 {
        self.tx.signature_hash()
    }
}
