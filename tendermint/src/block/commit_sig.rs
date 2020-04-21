//! CommitSig within Commit

use crate::serializers;
use crate::{account, Signature, Time};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

/// BlockIDFlag is used to indicate the validator has voted either for nil, a particular BlockID or was absent.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BlockIDFlag {
    /// BlockIDFlagAbsent - no vote was received from a validator.
    BlockIDFlagAbsent = 1,
    /// BlockIDFlagCommit - voted for the Commit.BlockID.
    BlockIDFlagCommit = 2,
    /// BlockIDFlagNil - voted for nil.
    BlockIDFlagNil = 3,
}

impl BlockIDFlag {
    /// Deserialize this type from a byte
    pub fn from_u8(byte: u8) -> Option<BlockIDFlag> {
        match byte {
            1 => Some(BlockIDFlag::BlockIDFlagAbsent),
            2 => Some(BlockIDFlag::BlockIDFlagCommit),
            3 => Some(BlockIDFlag::BlockIDFlagNil),
            _ => None,
        }
    }

    /// Serialize this type as a byte
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Serialize this type as a 32-bit unsigned integer
    pub fn to_u32(self) -> u32 {
        self as u32
    }
}

impl Serialize for BlockIDFlag {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_u8().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BlockIDFlag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let byte = u8::deserialize(deserializer)?;
        BlockIDFlag::from_u8(byte)
            .ok_or_else(|| D::Error::custom(format!("invalid block ID flag: {}", byte)))
    }
}

/// CommitSig represents a signature of a validator.
/// It's a part of the Commit and can be used to reconstruct the vote set given the validator set.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CommitSig {
    /// Block ID FLag
    pub block_id_flag: BlockIDFlag,

    /// Validator address
    #[serde(deserialize_with = "serializers::parse_non_empty_id")]
    pub validator_address: Option<account::Id>,

    /// Timestamp
    pub timestamp: Time,

    /// Signature
    #[serde(deserialize_with = "serializers::parse_non_empty_signature")]
    pub signature: Option<Signature>,
}

impl CommitSig {
    /// Checks if a validator's vote is absent
    pub fn is_absent(&self) -> bool {
        self.block_id_flag == BlockIDFlag::BlockIDFlagAbsent
    }
}
