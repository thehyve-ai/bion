use crate::core::transaction::pooled::PooledTransactionsElement;
use alloy_rlp::{Decodable, Encodable};
use libp2p::PeerId;
use tokio::sync::oneshot::Sender;

use super::{AppRequestType, ErrorCode, PeerRequestId, Request, Response};

#[derive(Debug)]
pub enum InboundMessage {
    InboundRequest {
        peer_id: PeerId,
        peer_request_id: PeerRequestId,
        request: Request,
    },
}

/// A wrapper for messages send to the P2P network service.
#[derive(Debug)]
pub enum OutboundMessage {
    /// Send an RPC request to the libp2p service.
    SendRequest {
        peer_id: PeerId,
        request: Request,
        request_type: AppRequestType,
        response_channel: Sender<Result<Response, ErrorCode>>,
    },
    /// Send a successful Response to the libp2p service.
    SendResponse {
        peer_id: PeerId,
        response: Response,
        peer_request_id: PeerRequestId,
    },

    /// Sends an error response to an RPC request.
    SendErrorResponse {
        peer_id: PeerId,
        error_code: ErrorCode,
        reason: String,
        id: PeerRequestId,
    },

    /// Sends a gossip message.
    SendGossipMessage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PooledTransactions(
    /// The transaction bodies, each of which should correspond to a requested hash.
    pub Vec<PooledTransactionsElement>,
);

impl Encodable for PooledTransactions {
    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.0.encode(out)
    }
}

impl Decodable for PooledTransactions {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self(Vec::<PooledTransactionsElement>::decode(buf)?))
    }
}
