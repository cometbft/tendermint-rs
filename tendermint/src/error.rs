//! Error types

use anomaly::{BoxError, Context};
use thiserror::Error;

/// Error type
pub type Error = BoxError;

/// Kinds of errors
#[derive(Clone, Eq, PartialEq, Debug, Error)]
pub enum Kind {
    /// Cryptographic operation failed
    #[error("cryptographic error")]
    Crypto,

    /// Malformatted or otherwise invalid cryptographic key
    #[error("invalid key")]
    InvalidKey,

    /// Input/output error
    #[error("I/O error")]
    Io,

    /// Length incorrect or too long
    #[error("length error")]
    Length,

    /// Parse error
    #[error("parse error")]
    Parse,

    /// Network protocol-related errors
    #[error("protocol error")]
    Protocol,

    /// Value out-of-range
    #[error("value out of range")]
    OutOfRange,

    /// Signature invalid
    #[error("bad signature")]
    SignatureInvalid,

    /// invalid message type
    #[error("invalid message type")]
    InvalidMessageType,

    /// Negative block height
    #[error("negative height")]
    NegativeHeight,

    /// Negative voting round
    #[error("negative round")]
    NegativeRound,

    /// Negative POL round
    #[error("negative POL round")]
    NegativePolRound,

    /// Negative validator index in vote
    #[error("negative validator index")]
    NegativeValidatorIndex,

    /// Invalid hash size in part_set_header
    #[error("invalid hash: expected hash size to be 32 bytes")]
    InvalidHashSize,

    /// No timestamp in vote
    #[error("no timestamp")]
    NoTimestamp,

    /// Invalid account ID length
    #[error("invalid account ID length")]
    InvalidAccountIdLength,

    /// Invalid signature ID length
    #[error("invalid signature ID length")]
    InvalidSignatureIdLength,

    /// Overflow during conversion
    #[error("integer overflow")]
    IntegerOverflow,

    /// No Vote found during conversion
    #[error("no vote found")]
    NoVoteFound,

    /// No Proposal found during conversion
    #[error("no proposal found")]
    NoProposalFound,

    /// Invalid AppHash length found during conversion
    #[error("invalid app hash Length")]
    InvalidAppHashLength,

    /// Invalid PartSetHeader
    #[error("invalid part set header")]
    InvalidPartSetHeader,
}

impl Kind {
    /// Add additional context.
    pub fn context(self, source: impl Into<BoxError>) -> Context<Kind> {
        Context::new(self, Some(source.into()))
    }
}
