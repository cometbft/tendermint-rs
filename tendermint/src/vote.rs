//! Votes from validators

mod canonical_vote;
mod power;
mod sign_vote;
mod validator_index;

pub use self::canonical_vote::CanonicalVote;
pub use self::power::Power;
pub use self::sign_vote::*;
pub use self::validator_index::ValidatorIndex;
use crate::chain::Id as ChainId;
use crate::consensus::State;
use crate::hash;
use crate::{account, block, Signature, Time};
use crate::{Error, Kind::*};
use bytes::BufMut;
use ed25519::Signature as ed25519Signature;
use ed25519::SIGNATURE_LENGTH as ed25519SignatureLength;
use std::convert::{TryFrom, TryInto};
use tendermint_proto::types::Vote as RawVote;
use tendermint_proto::{DomainType, Error as DomainTypeError};
use {
    crate::serializers,
    serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer},
};

use crate::signature::Signature::Ed25519;

/// Votes are signed messages from validators for a particular block which
/// include information about the validator signing it.
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#vote>
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Vote {
    /// Type of vote (prevote or precommit)
    #[serde(rename = "type")]
    pub vote_type: Type,

    /// Block height
    pub height: block::Height,

    /// Round
    #[serde(with = "serializers::from_str")]
    pub round: block::Round,

    /// Block ID
    #[serde(deserialize_with = "serializers::parse_non_empty_block_id")]
    pub block_id: Option<block::Id>,

    /// Timestamp
    pub timestamp: Option<Time>,

    /// Validator address
    pub validator_address: account::Id,

    /// Validator index
    #[serde(with = "serializers::from_str")]
    pub validator_index: ValidatorIndex,

    /// Signature
    pub signature: Signature,
}

impl DomainType<RawVote> for Vote {}

impl TryFrom<RawVote> for Vote {
    type Error = Error;

    fn try_from(value: RawVote) -> Result<Self, Self::Error> {
        if value.timestamp.is_none() {
            return Err(NoTimestamp.into());
        }
        Ok(Vote {
            vote_type: value.r#type.try_into()?,
            height: value.height.try_into()?,
            round: value.round.try_into()?,
            block_id: value.block_id.map(TryInto::try_into).transpose()?,
            timestamp: value.timestamp.map(TryInto::try_into).transpose()?,
            validator_address: value.validator_address.try_into()?,
            validator_index: value.validator_index.try_into()?,
            signature: value.signature.try_into()?,
        })
    }
}

impl From<Vote> for RawVote {
    fn from(value: Vote) -> Self {
        RawVote {
            r#type: value.vote_type.into(),
            height: value.height.into(),
            round: value.round.into(),
            block_id: value.block_id.map(Into::into),
            timestamp: value.timestamp.map(Into::into),
            validator_address: value.validator_address.into(),
            validator_index: value.validator_index.into(),
            signature: value.signature.into(),
        }
    }
}

impl Vote {
    /// Is this vote a prevote?
    pub fn is_prevote(&self) -> bool {
        match self.vote_type {
            Type::Prevote => true,
            Type::Precommit => false,
        }
    }

    /// Is this vote a precommit?
    pub fn is_precommit(&self) -> bool {
        match self.vote_type {
            Type::Precommit => true,
            Type::Prevote => false,
        }
    }

    /// Returns block_id.hash
    pub fn header_hash(&self) -> Option<hash::Hash> {
        self.block_id.clone().map(|b| b.hash)
    }

    /// Create signable bytes from Vote.
    pub fn to_signable_bytes<B>(
        &self,
        chain_id: ChainId,
        sign_bytes: &mut B,
    ) -> Result<bool, DomainTypeError>
    where
        B: BufMut,
    {
        CanonicalVote::new(self.clone(), chain_id).encode_length_delimited(sign_bytes)?;
        Ok(true)
    }

    /// Create signable vector from Vote.
    pub fn to_signable_vec(&self, chain_id: ChainId) -> Result<Vec<u8>, DomainTypeError> {
        CanonicalVote::new(self.clone(), chain_id).encode_length_delimited_vec()
    }

    /// Consensus state from this vote - This doesn't seem to be used anywhere.
    #[deprecated(
        since = "0.17.0",
        note = "This seems unnecessary, please raise it to the team, if you need it."
    )]
    pub fn consensus_state(&self) -> State {
        State {
            height: self.height,
            round: self.round,
            step: 6,
            block_id: self.block_id,
        }
    }
}

/// Default trait. Used in tests.
impl Default for Vote {
    fn default() -> Self {
        Vote {
            vote_type: Type::Prevote,
            height: Default::default(),
            round: Default::default(),
            block_id: None,
            timestamp: None,
            validator_address: account::Id::new([0; account::LENGTH]),
            validator_index: ValidatorIndex::try_from(0_i32).unwrap(),
            signature: Ed25519(ed25519Signature::new([0; ed25519SignatureLength])),
        }
    }
}
/// SignedVote is the union of a canonicalized vote, the signature on
/// the sign bytes of that vote and the id of the validator who signed it.
pub struct SignedVote {
    vote: CanonicalVote,
    validator_address: account::Id,
    signature: Signature,
}

impl SignedVote {
    /// Create new SignedVote from provided canonicalized vote, validator id, and
    /// the signature of that validator.
    pub fn new(
        vote: Vote,
        chain_id: ChainId,
        validator_address: account::Id,
        signature: Signature,
    ) -> SignedVote {
        let canonical_vote = CanonicalVote::new(vote, chain_id);
        SignedVote {
            vote: canonical_vote,
            signature,
            validator_address,
        }
    }

    /// Return the id of the validator that signed this vote.
    pub fn validator_id(&self) -> account::Id {
        self.validator_address
    }

    /// Return the bytes (of the canonicalized vote) that were signed.
    pub fn sign_bytes(&self) -> Vec<u8> {
        self.vote.encode_length_delimited_vec().unwrap()
    }

    /// Return the actual signature on the canonicalized vote.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

/// Types of votes
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    /// Votes for blocks which validators observe are valid for a given round
    Prevote = 1,

    /// Votes to commit to a particular block for a given round
    Precommit = 2,
}

impl DomainType<i32> for Type {}

impl TryFrom<i32> for Type {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Type::Prevote),
            2 => Ok(Type::Precommit),
            _ => Err(InvalidMessageType.into()),
        }
    }
}

impl From<Type> for i32 {
    fn from(value: Type) -> Self {
        value as i32
    }
}

impl Serialize for Type {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        i32::from(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let byte = i32::deserialize(deserializer)?;
        Type::try_from(byte).map_err(|_| D::Error::custom(format!("invalid vote type: {}", byte)))
    }
}
