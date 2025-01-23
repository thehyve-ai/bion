use alloy_eips::eip2718::{Decodable2718, Eip2718Result, Encodable2718};
use alloy_primitives::{Address, Signature, B256};
use alloy_rlp::{Buf, BufMut, Decodable, Encodable, Header};
use derive_more::derive::{AsRef, Deref};

use crate::aliases::TransactionHash;
use crate::core::signature::recover_signer;
use crate::p2p::HandleMempoolData;

use super::traits::SignableTransaction;
use super::{
    ShardsSidecar, Transaction, TransactionSigned, TxShardsWithSidecar, SHARD_TRANSACTION_TYPE,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PooledTransactionsElement {
    ShardTransaction(ShardTransaction),
}

impl PooledTransactionsElement {
    pub fn hash(&self) -> &TransactionHash {
        match self {
            PooledTransactionsElement::ShardTransaction(tx) => &tx.hash,
        }
    }

    /// Returns the signature as an alloy primitives signature
    pub fn signature(&self) -> Signature {
        match self {
            PooledTransactionsElement::ShardTransaction(tx) => tx.signature,
        }
    }

    pub fn signature_hash(&self) -> B256 {
        match self {
            PooledTransactionsElement::ShardTransaction(tx) => tx.transaction.signature_hash(),
        }
    }

    /// Recover signer from signature and hash.
    ///
    /// Returns `None` if the transaction's signature is invalid, see also [`Self::recover_signer`].
    pub fn recover_signer(&self) -> Option<Address> {
        recover_signer(&self.signature(), self.signature_hash())
    }

    /// Tries to recover signer and return [`PooledTransactionsElementEcRecovered`].
    ///
    /// Returns `Err(Self)` if the transaction's signature is invalid, see also
    /// [`Self::recover_signer`].
    pub fn try_into_ecrecovered(self) -> Result<PooledTransactionsElementEcRecovered, Self> {
        match self.recover_signer() {
            None => Err(self),
            Some(signer) => Ok(PooledTransactionsElementEcRecovered {
                transaction: self,
                signer,
            }),
        }
    }
}

impl Encodable for PooledTransactionsElement {
    fn length(&self) -> usize {
        let mut payload_length = self.encode_2718_len();

        payload_length += Header {
            list: false,
            payload_length,
        }
        .length();
        payload_length
    }

    fn encode(&self, out: &mut dyn BufMut) {
        self.network_encode(out);
    }
}

impl Decodable for PooledTransactionsElement {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        if buf.is_empty() {
            return Err(alloy_rlp::Error::InputTooShort);
        }

        let header = Header::decode(buf)?;

        // decode the type byte, only decode BlobTransaction if it is a 4844 transaction
        let tx_type = *buf.first().ok_or(alloy_rlp::Error::InputTooShort)?;
        let remaining_len = buf.len();

        // Advance the buffer past the type byte
        buf.advance(1);

        let tx = Self::typed_decode(tx_type, buf).map_err(alloy_rlp::Error::from)?;

        // check that the bytes consumed match the payload length
        let bytes_consumed = remaining_len - buf.len();
        if bytes_consumed != header.payload_length {
            return Err(alloy_rlp::Error::UnexpectedLength);
        }

        Ok(tx)
    }
}

impl Encodable2718 for PooledTransactionsElement {
    fn type_flag(&self) -> Option<u8> {
        match self {
            Self::ShardTransaction(_) => Some(SHARD_TRANSACTION_TYPE),
        }
    }

    fn encode_2718_len(&self) -> usize {
        match self {
            Self::ShardTransaction(shard_tx) => shard_tx.payload_len_with_type(false),
        }
    }

    fn encode_2718(&self, out: &mut dyn alloy_rlp::BufMut) {
        match self {
            Self::ShardTransaction(shard_tx) => {
                shard_tx.encode_with_type_inner(out, false);
            }
        }
    }
}

impl Decodable2718 for PooledTransactionsElement {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> Eip2718Result<Self> {
        match ty {
            SHARD_TRANSACTION_TYPE => {
                let shard_tx = ShardTransaction::decode_inner(buf)?;
                Ok(Self::ShardTransaction(shard_tx))
            }
            _ => Err(alloy_eips::eip2718::Eip2718Error::UnexpectedType(ty)),
        }
    }

    fn fallback_decode(_: &mut &[u8]) -> Eip2718Result<Self> {
        unreachable!()
    }
}

/// A transaction in the pool that contains a shard as a sidecar. We implement this as a
/// separate container to follow the specs of Ethereum 2.0.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShardTransaction {
    /// The transaction hash.
    pub hash: TransactionHash,
    /// The transaction signature.
    pub signature: Signature,
    /// The transaction payload with the sidecar.
    ///
    /// Note: this should flatten, but not possible with szz...
    pub transaction: TxShardsWithSidecar,
}

impl ShardTransaction {
    pub fn into_parts(self) -> (TransactionSigned, ShardsSidecar) {
        let transaction = TransactionSigned {
            transaction: Transaction::Shards(self.transaction.tx),
            hash: self.hash,
            signature: self.signature,
        };

        (transaction, self.transaction.sidecar)
    }

    /// Decodes a [`BlobTransaction`] from RLP. This expects the encoding to be:
    /// `rlp([transaction_payload_body, blobs, commitments, proofs])`
    ///
    /// where `transaction_payload_body` is a list:
    /// `[chain_id, nonce, max_priority_fee_per_gas, ..., y_parity, r, s]`
    ///
    /// Note: this should be used only when implementing other RLP decoding methods, and does not
    /// represent the full RLP decoding of the `PooledTransactionsElement` type.
    pub(crate) fn decode_inner(data: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let (transaction, signature, hash) =
            TxShardsWithSidecar::decode_signed_fields(data)?.into_parts();

        Ok(Self {
            transaction,
            hash,
            signature,
        })
    }

    /// Encodes the [`BlobTransaction`] fields as RLP, with a tx type. If `with_header` is `false`,
    /// the following will be encoded:
    /// `tx_type (0x03) || rlp([transaction_payload_body, blobs, commitments, proofs])`
    ///
    /// If `with_header` is `true`, the following will be encoded:
    /// `rlp(tx_type (0x03) || rlp([transaction_payload_body, blobs, commitments, proofs]))`
    ///
    /// NOTE: The header will be a byte string header, not a list header.
    pub(crate) fn encode_with_type_inner(&self, out: &mut dyn BufMut, with_header: bool) {
        // Calculate the length of:
        // `tx_type || rlp([transaction_payload_body, blobs, commitments, proofs])`
        //
        // to construct and encode the string header
        if with_header {
            Header {
                list: false,
                // add one for the tx type
                payload_length: 1 + self.payload_len(),
            }
            .encode(out);
        }

        out.put_u8(SHARD_TRANSACTION_TYPE);

        // Now we encode the inner blob transaction:
        self.encode_inner(out);
    }

    /// Encodes the [`BlobTransaction`] fields as RLP, with the following format:
    /// `rlp([transaction_payload_body, blobs, commitments, proofs])`
    ///
    /// where `transaction_payload_body` is a list:
    /// `[chain_id, nonce, max_priority_fee_per_gas, ..., y_parity, r, s]`
    ///
    /// Note: this should be used only when implementing other RLP encoding methods, and does not
    /// represent the full RLP encoding of the blob transaction.
    pub(crate) fn encode_inner(&self, out: &mut dyn BufMut) {
        self.transaction
            .encode_with_signature_fields(&self.signature, out);
    }

    /// Outputs the length of the RLP encoding of the blob transaction, including the tx type byte,
    /// optionally including the length of a wrapping string header. If `with_header` is `false`,
    /// the length of the following will be calculated:
    /// `tx_type (0x03) || rlp([transaction_payload_body, blobs, commitments, proofs])`
    ///
    /// If `with_header` is `true`, the length of the following will be calculated:
    /// `rlp(tx_type (0x03) || rlp([transaction_payload_body, blobs, commitments, proofs]))`
    pub(crate) fn payload_len_with_type(&self, with_header: bool) -> usize {
        if with_header {
            // Construct a header and use that to calculate the total length
            let wrapped_header = Header {
                list: false,
                // add one for the tx type byte
                payload_length: 1 + self.payload_len(),
            };

            // The total length is now the length of the header plus the length of the payload
            // (which includes the tx type byte)
            wrapped_header.length() + wrapped_header.payload_length
        } else {
            // Just add the length of the tx type to the payload length
            1 + self.payload_len()
        }
    }

    /// Outputs the length of the RLP encoding of the blob transaction with the following format:
    /// `rlp([transaction_payload_body, blobs, commitments, proofs])`
    ///
    /// where `transaction_payload_body` is a list:
    /// `[chain_id, nonce, max_priority_fee_per_gas, ..., y_parity, r, s]`
    ///
    /// Note: this should be used only when implementing other RLP encoding length methods, and
    /// does not represent the full RLP encoding of the blob transaction.
    pub(crate) fn payload_len(&self) -> usize {
        // The `transaction_payload_body` length is the length of the fields, plus the length of
        // its list header.
        let tx_header = Header {
            list: true,
            payload_length: self.transaction.tx.fields_len() + self.signature.rlp_vrs_len(),
        };

        let tx_length = tx_header.length() + tx_header.payload_length;

        // The payload length is the length of the `tranascation_payload_body` list, plus the
        // length of the blobs, commitments, and proofs.
        let payload_length = tx_length + self.transaction.sidecar.rlp_encoded_fields_length();

        // We use the calculated payload len to construct the first list header, which encompasses
        // everything in the tx - the length of the second, inner list header is part of
        // payload_length
        let blob_tx_header = Header {
            list: true,
            payload_length,
        };

        // The final length is the length of:
        //  * the outer blob tx header +
        //  * the inner tx header +
        //  * the inner tx fields +
        //  * the signature fields +
        //  * the sidecar fields
        blob_tx_header.length() + blob_tx_header.payload_length
    }
}

impl HandleMempoolData for Vec<PooledTransactionsElement> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn retain_by_hash(&mut self, mut f: impl FnMut(&TransactionHash) -> bool) {
        self.retain(|tx| f(tx.hash()))
    }
}

/// A signed pooled transaction with recovered signer.
#[derive(Debug, Clone, PartialEq, Eq, AsRef, Deref)]
pub struct PooledTransactionsElementEcRecovered {
    /// Signer of the transaction
    signer: Address,
    /// Signed transaction
    #[deref]
    #[as_ref]
    transaction: PooledTransactionsElement,
}

impl PooledTransactionsElementEcRecovered {
    pub fn from_components(transaction: PooledTransactionsElement, signer: Address) -> Self {
        Self {
            transaction,
            signer,
        }
    }

    /// Dissolve Self to its component
    pub fn into_components(self) -> (PooledTransactionsElement, Address) {
        (self.transaction, self.signer)
    }
}
