//! Types for RPC methods (requests and responses) in p2p.

use std::fmt::Display;

use alloy_primitives::{Bytes, B256};
use alloy_rlp::{Decodable, Encodable};
use libp2p::swarm::ConnectionId;
use tree_hash::TreeHash;

use crate::core::{transaction::blob::BlobShard, TransactionId, TransactionV0};

use super::PooledTransactions;

/// Identifier of inbound and outbound substreams from the handler's perspective.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct SubstreamId(usize);

impl Display for SubstreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "substream_id: {}", self.0)
    }
}

impl SubstreamId {
    pub fn new(id: usize) -> Self {
        SubstreamId(id)
    }

    pub fn incr(&mut self) {
        self.0 += 1;
    }
}

/// Identifier of requests sent by a peer.
pub type PeerRequestId = (ConnectionId, SubstreamId);

/// Type that identifies requests on the DA RPC.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

/// Type of application request.
#[derive(Debug, Clone, Copy)]
pub enum AppRequestType {
    /// Request was sent by the router.
    Router,

    /// Request was sent by the collector.
    Collector,
}

/// Type of request.
#[derive(Debug, Clone, Copy)]
pub enum RequestType {
    /// Application request.
    Application(AppRequestType),

    /// Internal request.
    Internal,
}

/// A wrapper of the lower-level request types for the RPC.
#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    Transactions(PooledTransactions),
}

/// A wrapper of the response types for the RPC.
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// Response to the `submit_shards` RPC method.
    Hashes(Vec<B256>),
}

/// Request type of the `submit_shards` RPC method.
#[derive(Debug, Clone, PartialEq)]
pub struct SubmitShardsRequest {
    /// The information about the transaction.
    pub transaction: TransactionV0,

    /// The shards of the blob.
    pub shards: Vec<BlobShard>,

    /// The total number of shards for the entire blob.
    /// Used for verification purposes.
    pub total_shards: u64,

    /// The signature of the transaction in bytes.
    pub signature: Bytes,
}

impl Encodable for SubmitShardsRequest {
    fn length(&self) -> usize {
        self.transaction.length() + self.shards.length() + self.total_shards.length()
    }

    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.transaction.encode(out);
        self.shards.encode(out);
        self.total_shards.encode(out);
    }
}

impl Decodable for SubmitShardsRequest {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self {
            transaction: Decodable::decode(buf)?,
            shards: Decodable::decode(buf)?,
            total_shards: Decodable::decode(buf)?,
            signature: Decodable::decode(buf)?,
        })
    }
}

impl SubmitShardsRequest {
    pub fn transaction_id(&self) -> TransactionId {
        self.transaction.tree_hash_root().into()
    }

    pub fn set_shards(&mut self, shards: Vec<BlobShard>) {
        self.shards = shards;
    }

    pub fn push_shard(&mut self, shard: BlobShard) {
        self.shards.push(shard);
    }
}
