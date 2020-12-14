//! `/tx_search` endpoint JSON-RPC wrapper

use crate::{Method, Order};
use serde::{Deserialize, Serialize};
use tendermint::{abci, block};
use tendermint_proto::types::TxProof;

/// Request for searching for transactions with their results.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    pub query: String,
    pub prove: bool,
    #[serde(with = "tendermint_proto::serializers::from_str")]
    pub page: u32,
    #[serde(with = "tendermint_proto::serializers::from_str")]
    pub per_page: u8,
    pub order_by: Order,
}

impl Request {
    /// Constructor.
    pub fn new(
        query: impl ToString,
        prove: bool,
        page: u32,
        per_page: u8,
        order_by: Order,
    ) -> Self {
        Self {
            query: query.to_string(),
            prove,
            page,
            per_page,
            order_by,
        }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> Method {
        Method::TxSearch
    }
}

impl crate::SimpleRequest for Request {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    pub txs: Vec<ResultTx>,
    #[serde(with = "tendermint_proto::serializers::from_str")]
    pub total_count: u32,
}

impl crate::Response for Response {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResultTx {
    pub hash: abci::transaction::Hash,
    pub height: block::Height,
    pub index: u32,
    pub tx_result: abci::DeliverTx,
    pub tx: abci::Transaction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<TxProof>,
}
