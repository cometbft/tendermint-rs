//! `/broadcast_tx_async`: broadcast a transaction and return immediately.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tendermint::{abci::Code, Hash};

use crate::{prelude::*, serializers};

/// `/broadcast_tx_async`: broadcast a transaction and return immediately.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Transaction to broadcast
    #[serde(with = "serializers::bytes::base64string")]
    pub tx: Vec<u8>,
}

impl Request {
    /// Create a new async transaction broadcast RPC request
    pub fn new(tx: impl Into<Vec<u8>>) -> Request {
        Request { tx: tx.into() }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::BroadcastTxAsync
    }
}

impl crate::SimpleRequest for Request {}

/// Response from either an async or sync transaction broadcast request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Code
    pub code: Code,

    /// Data
    #[serde(with = "serializers::bytes::base64string")]
    pub data: Bytes,

    /// Log
    pub log: String,

    /// Transaction hash
    pub hash: Hash,
}

impl crate::Response for Response {}
