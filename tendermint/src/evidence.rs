//! Evidence of malfeasance by validators (i.e. signing conflicting votes).

use core::{
    convert::{TryFrom, TryInto},
    slice,
};

use serde::{Deserialize, Serialize};
use tendermint_proto::google::protobuf::Duration as RawDuration;
use tendermint_proto::v0_37::types::Evidence as RawEvidence;
use tendermint_proto::Protobuf;

use crate::{error::Error, prelude::*, serializers, vote::Power, Time, Vote};

/// Evidence of malfeasance by validators (i.e. signing conflicting votes).
/// encoded using an Amino prefix. There is currently only a single type of
/// evidence: `DuplicateVoteEvidence`.
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#evidence>
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawEvidence", into = "RawEvidence")] // Used by RPC /broadcast_evidence endpoint
#[allow(clippy::large_enum_variant)]
pub enum Evidence {
    /// Duplicate vote evidence
    DuplicateVote(DuplicateVoteEvidence),

    /// LightClient attack evidence - Todo: Implement details
    LightClientAttackEvidence,
}

/// Duplicate vote evidence
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuplicateVoteEvidence {
    pub vote_a: Vote,
    pub vote_b: Vote,
    pub total_voting_power: Power,
    pub validator_power: Power,
    pub timestamp: Time,
}

impl DuplicateVoteEvidence {
    /// constructor
    pub fn new(vote_a: Vote, vote_b: Vote) -> Result<Self, Error> {
        if vote_a.height != vote_b.height {
            return Err(Error::invalid_evidence());
        }

        // Todo: make more assumptions about what is considered a valid evidence for duplicate vote
        Ok(Self {
            vote_a,
            vote_b,
            total_voting_power: Default::default(),
            validator_power: Default::default(),
            timestamp: Time::unix_epoch(),
        })
    }

    /// Get votes
    pub fn votes(&self) -> (&Vote, &Vote) {
        (&self.vote_a, &self.vote_b)
    }
}

/// A list of `Evidence`.
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#evidencedata>
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct List(Vec<Evidence>);

impl List {
    /// Create a new evidence data collection
    pub fn new<I>(into_evidence: I) -> List
    where
        I: Into<Vec<Evidence>>,
    {
        List(into_evidence.into())
    }

    /// Convert this evidence data into a vector
    pub fn into_vec(self) -> Vec<Evidence> {
        self.0
    }

    /// Iterate over the evidence data
    pub fn iter(&self) -> slice::Iter<'_, Evidence> {
        self.0.iter()
    }
}

impl AsRef<[Evidence]> for List {
    fn as_ref(&self) -> &[Evidence] {
        &self.0
    }
}

/// EvidenceParams determine how we handle evidence of malfeasance.
///
/// [Tendermint documentation](https://docs.tendermint.com/master/spec/core/data_structures.html#evidenceparams)
#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
// Todo: This struct is ready to be converted through tendermint_proto::types::EvidenceParams.
// https://github.com/informalsystems/tendermint-rs/issues/741
pub struct Params {
    /// Max age of evidence, in blocks.
    #[serde(with = "serializers::from_str")]
    pub max_age_num_blocks: u64,

    /// Max age of evidence, in time.
    ///
    /// It should correspond with an app's "unbonding period" or other similar
    /// mechanism for handling [Nothing-At-Stake attacks][nas].
    ///
    /// [nas]: https://github.com/ethereum/wiki/wiki/Proof-of-Stake-FAQ#what-is-the-nothing-at-stake-problem-and-how-can-it-be-fixed
    pub max_age_duration: Duration,

    /// This sets the maximum size of total evidence in bytes that can be
    /// committed in a single block, and should fall comfortably under the max
    /// block bytes. The default is 1048576 or 1MB.
    #[serde(with = "serializers::from_str", default)]
    pub max_bytes: i64,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use pb::types::{
        evidence::Sum as RawSum, DuplicateVoteEvidence as RawDuplicateVoteEvidence,
        Evidence as RawEvidence, EvidenceList as RawEvidenceList,
        EvidenceParams as RawEvidenceParams,
    };

    use super::{List, DuplicateVoteEvidence, Evidence, Params};
    use crate::{error::Error, prelude::*};

    impl Protobuf<RawEvidence> for Evidence {}

    impl TryFrom<RawEvidence> for Evidence {
        type Error = Error;

        fn try_from(message: RawEvidence) -> Result<Self, Self::Error> {
            use RawSum::*;
            match message.sum.ok_or_else(Error::invalid_evidence)? {
                DuplicateVoteEvidence(ev) => Ok(Evidence::DuplicateVote(ev.try_into()?)),
                LightClientAttackEvidence(_ev) => Ok(Evidence::LightClientAttackEvidence),
            }
        }
    }

    impl From<Evidence> for RawEvidence {
        fn from(value: Evidence) -> Self {
            let sum = match value {
                Evidence::DuplicateVote(ev) => Some(RawSum::DuplicateVoteEvidence(ev.into())),
                Evidence::LightClientAttackEvidence => None,
            };
            RawEvidence { sum }
        }
    }

    impl Protobuf<RawDuplicateVoteEvidence> for DuplicateVoteEvidence {}

    impl TryFrom<RawDuplicateVoteEvidence> for DuplicateVoteEvidence {
        type Error = Error;

        fn try_from(value: RawDuplicateVoteEvidence) -> Result<Self, Self::Error> {
            Ok(Self {
                vote_a: value
                    .vote_a
                    .ok_or_else(Error::missing_evidence)?
                    .try_into()?,
                vote_b: value
                    .vote_b
                    .ok_or_else(Error::missing_evidence)?
                    .try_into()?,
                total_voting_power: value.total_voting_power.try_into()?,
                validator_power: value.validator_power.try_into()?,
                timestamp: value
                    .timestamp
                    .ok_or_else(Error::missing_timestamp)?
                    .try_into()?,
            })
        }
    }

    impl From<DuplicateVoteEvidence> for RawDuplicateVoteEvidence {
        fn from(value: DuplicateVoteEvidence) -> Self {
            RawDuplicateVoteEvidence {
                vote_a: Some(value.vote_a.into()),
                vote_b: Some(value.vote_b.into()),
                total_voting_power: value.total_voting_power.into(),
                validator_power: value.total_voting_power.into(),
                timestamp: Some(value.timestamp.into()),
            }
        }
    }

    impl Protobuf<RawEvidenceList> for List {}

    impl TryFrom<RawEvidenceList> for List {
        type Error = Error;
        fn try_from(value: RawEvidenceList) -> Result<Self, Self::Error> {
            let evidence = value
                .evidence
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Self(evidence))
        }
    }

    impl From<List> for RawEvidenceList {
        fn from(value: List) -> Self {
            RawEvidenceList {
                evidence: value.0.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl Protobuf<RawEvidenceParams> for Params {}

    impl TryFrom<RawEvidenceParams> for Params {
        type Error = Error;

        fn try_from(value: RawEvidenceParams) -> Result<Self, Self::Error> {
            Ok(Self {
                max_age_num_blocks: value
                    .max_age_num_blocks
                    .try_into()
                    .map_err(Error::negative_max_age_num)?,
                max_age_duration: value
                    .max_age_duration
                    .ok_or_else(Error::missing_max_age_duration)?
                    .try_into()?,
                max_bytes: value.max_bytes,
            })
        }
    }

    impl From<Params> for RawEvidenceParams {
        fn from(value: Params) -> Self {
            Self {
                // Todo: Implement proper domain types so this becomes infallible
                max_age_num_blocks: value.max_age_num_blocks.try_into().unwrap(),
                max_age_duration: Some(value.max_age_duration.into()),
                max_bytes: value.max_bytes,
            }
        }
    }
}

/// Duration is a wrapper around core::time::Duration
/// essentially, to keep the usages look cleaner
/// i.e. you can avoid using serde annotations everywhere
/// Todo: harmonize google::protobuf::Duration, core::time::Duration and this. Too many structs.
/// <https://github.com/informalsystems/tendermint-rs/issues/741>
#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Duration(#[serde(with = "serializers::time_duration")] pub core::time::Duration);

impl From<Duration> for core::time::Duration {
    fn from(d: Duration) -> core::time::Duration {
        d.0
    }
}

impl Protobuf<RawDuration> for Duration {}

impl TryFrom<RawDuration> for Duration {
    type Error = Error;

    fn try_from(value: RawDuration) -> Result<Self, Self::Error> {
        Ok(Self(core::time::Duration::new(
            value.seconds.try_into().map_err(Error::integer_overflow)?,
            value.nanos.try_into().map_err(Error::integer_overflow)?,
        )))
    }
}

impl From<Duration> for RawDuration {
    fn from(value: Duration) -> Self {
        // Todo: make the struct into a proper domaintype so this becomes infallible.
        Self {
            seconds: value.0.as_secs() as i64,
            nanos: value.0.subsec_nanos() as i32,
        }
    }
}
