use std::fmt::Display;

use alloy_consensus::EncodableSignature;
use alloy_primitives::{hex::ToHexExt, keccak256, Address, FixedBytes, Parity, Signature, B256};
use alloy_rlp::{length_of_length, BufMut, Decodable, Encodable, Header};
use blob::BlobShard;
use derive_more::derive::{AsRef, Deref};
use lighthouse_bls::PublicKeyBytes;
use serde::{Deserialize, Serialize};
use signed::Signed;
use ssz_types::typenum::U256;
use traits::SignableTransaction;

use crate::{
    aliases::{BlobTTL, DacId, TransactionHash},
    consts::OPERATOR_PUB_KEY_LENGTH,
    kzg::KzgCommitment,
};

pub use self::v0::TransactionV0;

use super::attestation::Attestation;

pub mod blob;
pub mod pooled;
pub mod signed;
pub mod traits;
pub mod v0;

pub const SHARD_TRANSACTION_TYPE: u8 = 0x00;
pub const ATTESTATION_TRANSACTION_TYPE: u8 = 0x01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(B256);

impl From<&Attestation> for TransactionId {
    fn from(attestation: &Attestation) -> Self {
        Self(attestation.data.data)
    }
}

impl Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hex_string())
    }
}

impl TransactionId {
    pub fn new(id: B256) -> Self {
        Self(id)
    }

    pub fn id(&self) -> B256 {
        self.0
    }

    pub fn hex_string(&self) -> String {
        format!("0x{}", self.0.encode_hex())
    }
}

impl From<B256> for TransactionId {
    fn from(id: B256) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub struct TransactionSigned {
    pub hash: TransactionHash,

    pub signature: Signature,

    pub transaction: Transaction,
}

impl Encodable for TransactionSigned {
    fn encode(&self, out: &mut dyn BufMut) {
        self.hash.encode(out);
        self.signature.encode(out);
        self.transaction.encode(out);
    }
}

impl Decodable for TransactionSigned {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self {
            hash: Decodable::decode(buf)?,
            signature: Decodable::decode(buf)?,
            transaction: Decodable::decode(buf)?,
        })
    }
}

impl super::transaction::traits::Transaction for TransactionSigned {
    fn dac_id(&self) -> Option<DacId> {
        self.transaction.dac_id()
    }

    fn blob_versioned_hashes(&self) -> Option<&[B256]> {
        self.transaction.blob_versioned_hashes()
    }

    fn nonce(&self) -> u64 {
        self.transaction.nonce()
    }

    fn ty(&self) -> u8 {
        self.transaction.ty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Transaction {
    Shards(TxShards),
}

impl Encodable for Transaction {
    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        match self {
            Transaction::Shards(shards) => {
                out.put_u8(0);
                shards.encode_fields(out);
            }
        }
    }
}

impl Decodable for Transaction {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        if buf.len() < 2 {
            return Err(alloy_rlp::Error::InputTooShort);
        }

        let tag = buf[0];
        *buf = &buf[1..]; // Advance the buffer

        match tag {
            0 => {
                let shards = TxShards::decode_fields(buf)?;
                Ok(Transaction::Shards(shards))
            }
            _ => Err(alloy_rlp::Error::Custom(
                "Invalid tag for Transaction".into(),
            )),
        }
    }
}

impl self::traits::Transaction for Transaction {
    fn dac_id(&self) -> Option<DacId> {
        match self {
            Transaction::Shards(tx) => Some(tx.dac_id),
        }
    }

    fn blob_versioned_hashes(&self) -> Option<&[B256]> {
        match self {
            Transaction::Shards(tx) => Some(tx.blob_versioned_hashes.as_slice()),
        }
    }

    fn nonce(&self) -> u64 {
        match self {
            Transaction::Shards(tx) => tx.nonce,
        }
    }

    fn ty(&self) -> u8 {
        match self {
            Transaction::Shards(tx) => tx.ty(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]

pub struct TxShardsWithSidecar {
    pub tx: TxShards,

    pub sidecar: ShardsSidecar,
}

impl TxShardsWithSidecar {
    pub const fn from_tx_and_sidecar(tx: TxShards, sidecar: ShardsSidecar) -> Self {
        Self { tx, sidecar }
    }

    /// Decodes the transaction from RLP bytes, including the signature.
    ///
    /// This __does not__ expect the bytes to start with a transaction type byte or string
    /// header.
    ///
    /// This __does__ expect the bytes to start with a list header and include a signature.
    ///
    /// This is the inverse of [TxEip4844WithSidecar::encode_with_signature_fields].
    #[doc(hidden)]
    pub fn decode_signed_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Signed<Self>> {
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(alloy_rlp::Error::UnexpectedString);
        }

        // record original length so we can check encoding
        let original_len = buf.len();

        // decode the inner tx
        let inner_tx = TxShards::decode_signed_fields(buf)?;

        // decode the sidecar
        let sidecar = ShardsSidecar::rlp_decode_fields(buf)?;

        if buf.len() + header.payload_length != original_len {
            return Err(alloy_rlp::Error::ListLengthMismatch {
                expected: header.payload_length,
                got: original_len - buf.len(),
            });
        }

        let (tx, signature, hash) = inner_tx.into_parts();

        // create unchecked signed tx because these checks should have happened during construction
        // of `Signed<TxEip4844>` in `TxEip4844::decode_signed_fields`
        Ok(Signed::new_unchecked(
            Self::from_tx_and_sidecar(tx, sidecar),
            signature,
            hash,
        ))
    }

    /// Encodes the transaction from RLP bytes, including the signature. This __does not__ encode a
    /// tx type byte or string header.
    ///
    /// This __does__ encode a list header and include a signature.
    ///
    /// This encodes the following:
    /// `rlp([tx_payload, blobs, commitments, proofs])`
    ///
    /// where `tx_payload` is the RLP encoding of the [TxEip4844] transaction fields:
    /// `rlp([chain_id, nonce, max_priority_fee_per_gas, ..., v, r, s])`
    pub fn encode_with_signature_fields<S>(&self, signature: &S, out: &mut dyn BufMut)
    where
        S: EncodableSignature,
    {
        let inner_payload_length = self.tx.fields_len() + signature.rlp_vrs_len();
        let inner_header = Header {
            list: true,
            payload_length: inner_payload_length,
        };

        let outer_payload_length =
            inner_header.length() + inner_payload_length + self.sidecar.rlp_encoded_fields_length();
        let outer_header = Header {
            list: true,
            payload_length: outer_payload_length,
        };

        // write the two headers
        outer_header.encode(out);
        inner_header.encode(out);

        // now write the fields
        self.tx.encode_fields(out);
        signature.write_rlp_vrs(out);
        self.sidecar.rlp_encode_fields(out);
    }
}

impl traits::Transaction for TxShardsWithSidecar {
    fn dac_id(&self) -> Option<DacId> {
        Some(self.tx.dac_id)
    }

    fn blob_versioned_hashes(&self) -> Option<&[B256]> {
        Some(self.tx.blob_versioned_hashes.as_slice())
    }

    fn nonce(&self) -> u64 {
        self.tx.nonce
    }

    fn ty(&self) -> u8 {
        SHARD_TRANSACTION_TYPE
    }
}

pub type MaxShardsLen = U256;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct ShardsSidecar {
    /// The KZG commitment of the blob
    ///
    /// Todo: change this to an enum to enable alternative encoding schemes.
    pub commitment: KzgCommitment,

    /// The operator assigned to attest to the shard.
    pub operator: FixedBytes<OPERATOR_PUB_KEY_LENGTH>,
    /// The shards of the blob that are being submitted. Includes:
    /// - KZG Proof
    /// - Coefficients
    /// - Shard index (might remove)
    pub shard: BlobShard,
}

impl ShardsSidecar {
    /// RLP decode the fields of a [ShardsSidecar].
    pub fn rlp_decode_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self {
            commitment: Decodable::decode(buf)?,
            operator: Decodable::decode(buf)?,
            shard: Decodable::decode(buf)?,
        })
    }

    /// Outputs the RLP length of the [ShardsSidecar] fields, without
    /// a RLP header.
    #[doc(hidden)]
    pub fn rlp_encoded_fields_length(&self) -> usize {
        self.shard.length() + self.operator.length() + self.commitment.length()
    }

    /// RLP encode the fields of a [ShardsSidecar].
    #[inline]
    pub fn rlp_encode_fields(&self, out: &mut dyn BufMut) {
        // Encode the blobs, commitments, and proofs
        self.commitment.encode(out);
        self.operator.encode(out);
        self.shard.encode(out);
    }

    pub fn operator_pubkey(&self) -> Result<PublicKeyBytes, lighthouse_bls::Error> {
        PublicKeyBytes::deserialize(self.operator.as_slice())
    }
}

impl Decodable for ShardsSidecar {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Self::rlp_decode_fields(buf)
    }
}

impl Encodable for ShardsSidecar {
    fn encode(&self, out: &mut dyn BufMut) {
        self.rlp_encode_fields(out);
    }
}

/// The payload of a shards transaction. Can be interpreted as the 'header'
/// of the transaction.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TxShards {
    /// The identifier of the DAC
    pub dac_id: DacId,
    /// The transaction number as a scalar value
    pub nonce: u64,
    /// The time-to-live of the blob in seconds.
    pub ttl: BlobTTL,
    /// The size in symbols of the complete blob. This
    /// is necessary for the encoding parameters
    /// of the blob itself.
    ///
    /// If the `blob_length` does not match the size of
    /// the actual submitted blob, the encoding verification
    /// will fail and the transaction will be rejected.
    pub blob_length: u64,
    /// The commitment of a blob as a versioned hash. The versioned
    /// hash ensures that the blob is committed to a specific version.
    /// Currently version 0x00 is KZG.
    pub blob_versioned_hashes: Vec<B256>,
}

impl TxShards {
    pub const fn tx_type(&self) -> u8 {
        SHARD_TRANSACTION_TYPE
    }

    pub fn fields_len(&self) -> usize {
        let mut len = 0;
        len += self.dac_id.length();
        len += self.nonce.length();
        len += self.ttl.length();
        len += self.blob_length.length();
        len += self.blob_versioned_hashes.length();
        len
    }

    /// Encodes only the transaction's fields into the desired buffer, without a RLP header.
    pub(crate) fn encode_fields(&self, out: &mut dyn BufMut) {
        self.dac_id.encode(out);
        self.nonce.encode(out);
        self.ttl.encode(out);
        self.blob_length.encode(out);
        self.blob_versioned_hashes.encode(out);
    }

    /// Returns what the encoded length should be, if the transaction were RLP encoded with the
    /// given signature, depending on the value of `with_header`.
    ///
    /// If `with_header` is `true`, the payload length will include the RLP header length.
    /// If `with_header` is `false`, the payload length will not include the RLP header length.
    pub fn encoded_len_with_signature<S>(&self, signature: &S, with_header: bool) -> usize
    where
        S: EncodableSignature,
    {
        // this counts the tx fields and signature fields
        let payload_length = self.fields_len() + signature.rlp_vrs_len();

        // this counts:
        // * tx type byte
        // * inner header length
        // * inner payload length
        let inner_payload_length = 1
            + Header {
                list: true,
                payload_length,
            }
            .length()
            + payload_length;

        if with_header {
            // header length plus length of the above, wrapped with a string header
            Header {
                list: false,
                payload_length: inner_payload_length,
            }
            .length()
                + inner_payload_length
        } else {
            inner_payload_length
        }
    }

    /// Inner encoding function that is used for both rlp [`Encodable`] trait and for calculating
    /// hash that for eip2718 does not require a rlp header
    #[doc(hidden)]
    pub fn encode_with_signature<S>(&self, signature: &S, out: &mut dyn BufMut, with_header: bool)
    where
        S: EncodableSignature,
    {
        let payload_length = self.fields_len() + signature.rlp_vrs_len();
        if with_header {
            Header {
                list: false,
                payload_length: 1
                    + Header {
                        list: true,
                        payload_length,
                    }
                    .length()
                    + payload_length,
            }
            .encode(out);
        }
        out.put_u8(self.tx_type() as u8);
        self.encode_with_signature_fields(signature, out);
    }

    /// Encodes the transaction from RLP bytes, including the signature. This __does not__ encode a
    /// tx type byte or string header.
    ///
    /// This __does__ encode a list header and include a signature.
    pub fn encode_with_signature_fields<S>(&self, signature: &S, out: &mut dyn BufMut)
    where
        S: EncodableSignature,
    {
        let payload_length = self.fields_len() + signature.rlp_vrs_len();
        let header = Header {
            list: true,
            payload_length,
        };
        header.encode(out);
        self.encode_fields(out);
        signature.write_rlp_vrs(out);
    }

    pub fn decode_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self {
            dac_id: Decodable::decode(buf)?,
            nonce: Decodable::decode(buf)?,
            ttl: Decodable::decode(buf)?,
            blob_length: Decodable::decode(buf)?,
            blob_versioned_hashes: Decodable::decode(buf)?,
        })
    }

    pub fn encode_for_signing(&self, out: &mut dyn BufMut) {
        out.put_u8(self.tx_type() as u8);
        Header {
            list: true,
            payload_length: self.fields_len(),
        }
        .encode(out);
        self.encode_fields(out);
    }

    /// Outputs the length of the signature RLP encoding for the transaction.
    pub fn payload_len_for_signature(&self) -> usize {
        let payload_length = self.fields_len();
        // 'transaction type byte length' + 'header length' + 'payload length'
        1 + Header {
            list: true,
            payload_length,
        }
        .length()
            + payload_length
    }

    /// Decodes the transaction from RLP bytes, including the signature.
    ///
    /// This __does not__ expect the bytes to start with a transaction type byte or string
    /// header.
    ///
    /// This __does__ expect the bytes to start with a list header and include a signature.
    #[doc(hidden)]
    pub fn decode_signed_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Signed<Self>> {
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(alloy_rlp::Error::UnexpectedString);
        }

        // record original length so we can check encoding
        let original_len = buf.len();

        let tx = Self::decode_fields(buf)?;
        let signature = Signature::decode_rlp_vrs(buf)?;

        if !matches!(signature.v(), Parity::Parity(_)) {
            return Err(alloy_rlp::Error::Custom(
                "invalid parity for typed transaction",
            ));
        }

        let signed = tx.into_signed(signature);
        if buf.len() + header.payload_length != original_len {
            return Err(alloy_rlp::Error::ListLengthMismatch {
                expected: header.payload_length,
                got: original_len - buf.len(),
            });
        }

        Ok(signed)
    }
}

impl SignableTransaction<Signature> for TxShards {
    fn set_dac_id(&mut self, dac_id: DacId) {
        self.dac_id = dac_id;
    }

    fn encode_for_signing(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.encode_for_signing(out);
    }

    fn payload_len_for_signature(&self) -> usize {
        self.payload_len_for_signature()
    }

    fn into_signed(self, signature: Signature) -> Signed<Self> {
        // Drop any v chain id value to ensure the signature format is correct at the time of
        // combination for an EIP-4844 transaction. V should indicate the y-parity of the
        // signature.
        let signature = signature.with_parity_bool();

        let mut buf = Vec::with_capacity(self.encoded_len_with_signature(&signature, false));
        self.encode_with_signature(&signature, &mut buf, false);
        let hash = keccak256(&buf);

        Signed::new_unchecked(self, signature, hash)
    }
}

impl Encodable for TxShards {
    fn encode(&self, out: &mut dyn BufMut) {
        Header {
            list: true,
            payload_length: self.fields_len(),
        }
        .encode(out);
        self.encode_fields(out);
    }

    fn length(&self) -> usize {
        let payload_length = self.fields_len();
        length_of_length(payload_length) + payload_length
    }
}

impl Decodable for TxShards {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let header = Header::decode(buf)?;
        let remaining_len = buf.len();

        if header.payload_length > remaining_len {
            return Err(alloy_rlp::Error::InputTooShort);
        }

        Self::decode_fields(buf)
    }
}

impl self::traits::Transaction for TxShards {
    fn ty(&self) -> u8 {
        SHARD_TRANSACTION_TYPE
    }

    fn dac_id(&self) -> Option<DacId> {
        Some(self.dac_id)
    }

    fn blob_versioned_hashes(&self) -> Option<&[B256]> {
        Some(self.blob_versioned_hashes.as_slice())
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }
}

impl SignableTransaction<Signature> for TxShardsWithSidecar {
    fn set_dac_id(&mut self, dac_id: DacId) {
        self.tx.dac_id = dac_id;
    }

    fn encode_for_signing(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.tx.encode_for_signing(out);
    }

    fn payload_len_for_signature(&self) -> usize {
        self.tx.payload_len_for_signature()
    }

    fn into_signed(self, signature: Signature) -> Signed<Self, Signature>
    where
        Self: Sized,
    {
        // Drop any v chain id value to ensure the signature format is correct at the time of
        // combination for an EIP-4844 transaction. V should indicate the y-parity of the
        // signature.
        let signature = signature.with_parity_bool();

        let mut buf = Vec::with_capacity(self.tx.encoded_len_with_signature(&signature, false));
        // The sidecar is NOT included in the signed payload, only the transaction fields and the
        // type byte. Include the type byte.
        //
        // Include the transaction fields, making sure to __not__ use the sidecar, and __not__
        // encode a header.
        self.tx.encode_with_signature(&signature, &mut buf, false);
        let hash = keccak256(&buf);

        Signed::new_unchecked(self, signature, hash)
    }
}

// impl SignableTransaction<Signature> for TxShardsWithSidecar {
//     fn set_dac_id(&mut self, dac_id: DacId) {
//         self.tx.dac_id = dac_id;
//     }

//     fn encode_for_signing(&self, out: &mut Vec<u8>) {
//         self.tx.encode_for_signing(out);
//     }

//     fn
// }

/// Signed transaction with recovered signer.
#[derive(Debug, Clone, PartialEq, Hash, Eq, AsRef, Deref)]
pub struct TransactionSignedEcRecovered {
    /// Signer of the transaction
    signer: Address,
    /// Signed transaction
    #[deref]
    #[as_ref]
    signed_transaction: TransactionSigned,
}

impl Encodable for TransactionSignedEcRecovered {
    fn encode(&self, out: &mut dyn BufMut) {
        self.signer.encode(out);
        self.signed_transaction.encode(out);
    }
}

impl Decodable for TransactionSignedEcRecovered {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self {
            signer: Decodable::decode(buf)?,
            signed_transaction: Decodable::decode(buf)?,
        })
    }
}

impl TransactionSignedEcRecovered {
    /// Create [`TransactionSignedEcRecovered`] from [`TransactionSigned`] and [`Address`] of the
    /// signer.
    #[inline]
    pub const fn from_signed_transaction(
        signed_transaction: TransactionSigned,
        signer: Address,
    ) -> Self {
        Self {
            signed_transaction,
            signer,
        }
    }

    pub fn blob_length(&self) -> Option<u64> {
        match &self.signed_transaction.transaction {
            Transaction::Shards(tx) => Some(tx.blob_length),
        }
    }
}
