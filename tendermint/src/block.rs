//! Blocks within the chains of a Tendermint network

mod commit;
pub mod commit_sig;
pub mod header;
mod height;
mod id;
mod meta;
pub mod parts;
mod round;
pub mod signed_header;
mod size;

pub use self::{
    commit::*,
    commit_sig::*,
    header::Header,
    height::*,
    id::{Id, ParseId},
    meta::Meta,
    round::*,
    size::Size,
};
use crate::{abci::transaction, evidence, serializers};
use serde::{Deserialize, Deserializer, Serialize};

/// Blocks consist of a header, transactions, votes (the commit), and a list of
/// evidence of malfeasance (i.e. signing conflicting votes).
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#block>
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Block {
    /// Block header
    pub header: Header,

    /// Transaction data
    pub data: transaction::Data,

    /// Evidence of malfeasance
    pub evidence: evidence::Data,

    /// Last commit
    #[serde(deserialize_with = "parse_non_empty_commit")]
    pub last_commit: Option<Commit>,
}

pub(crate) fn parse_non_empty_commit<'de, D>(deserializer: D) -> Result<Option<Commit>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct TmpCommit {
        pub height: Height,
        pub round: u32,
        #[serde(deserialize_with = "serializers::parse_non_empty_block_id")]
        pub block_id: Option<Id>,
        pub signatures: Option<CommitSigs>,
    }

    if let Some(commit) = <Option<TmpCommit>>::deserialize(deserializer)? {
        if let Some(block_id) = commit.block_id {
            Ok(Some(Commit {
                height: commit.height,
                round: commit.round,
                block_id,
                signatures: commit.signatures.unwrap_or_else(|| CommitSigs::new(vec![])),
            }))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
