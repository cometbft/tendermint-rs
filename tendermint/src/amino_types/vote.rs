use super::{
    block_id::{BlockId, CanonicalBlockId, CanonicalPartSetHeader},
    signature::SignableMsg,
    validate,
    validate::{ConsensusMessage, Kind::*},
    SignedMsgType,
};
use crate::amino_types::PartSetHeader;
use crate::{
    block::{self, ParseId},
    chain, consensus,
    error::Error,
    vote,
};
use bytes::BufMut;
use prost::{EncodeError, Message};
use prost_types::Timestamp;
use std::convert::TryFrom;
use tendermint_proto::privval::RemoteSignerError;

const VALIDATOR_ADDR_SIZE: usize = 20;

// Copied from tendermint_proto::types::Vote
/// Vote represents a prevote, precommit, or commit vote from validators for
/// consensus.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    #[prost(enumeration = "SignedMsgType", tag = "1")]
    pub r#type: i32,
    #[prost(int64, tag = "2")]
    pub height: i64,
    #[prost(int32, tag = "3")]
    pub round: i32,
    /// zero if vote is nil.
    #[prost(message, optional, tag = "4")]
    pub block_id: ::std::option::Option<BlockId>,
    #[prost(message, optional, tag = "5")]
    pub timestamp: ::std::option::Option<::prost_types::Timestamp>,
    #[prost(bytes, tag = "6")]
    pub validator_address: Vec<u8>,
    #[prost(int32, tag = "7")]
    pub validator_index: i32,
    #[prost(bytes, tag = "8")]
    pub signature: Vec<u8>,
}

impl Vote {
    fn msg_type(&self) -> Option<SignedMsgType> {
        if self.r#type == SignedMsgType::Prevote {
            Some(SignedMsgType::Prevote)
        } else if self.r#type == SignedMsgType::Precommit {
            Some(SignedMsgType::Precommit)
        } else {
            None
        }
    }
}

impl From<&vote::Vote> for Vote {
    fn from(vote: &vote::Vote) -> Self {
        Vote {
            r#type: vote.vote_type.to_u32() as i32,
            height: vote.height.value() as i64, // TODO potential overflow :-/
            round: vote.round as i32,
            block_id: vote.block_id.as_ref().map(|block_id| BlockId {
                hash: block_id.hash.as_bytes().to_vec(),
                part_set_header: block_id.parts.as_ref().map(PartSetHeader::from),
            }),
            timestamp: Some(Timestamp::from(vote.timestamp.to_system_time().unwrap())),
            validator_address: vote.validator_address.as_bytes().to_vec(),
            validator_index: vote.validator_index as i32, // TODO potential overflow :-/
            signature: vote.signature.as_bytes().to_vec(),
        }
    }
}

impl block::ParseHeight for Vote {
    fn parse_block_height(&self) -> Result<block::Height, Error> {
        block::Height::try_from(self.height)
    }
}

// Copied from tendermint_proto::privval::SignVoteRequest
/// SignVoteRequest is a request to sign a vote
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignVoteRequest {
    #[prost(message, optional, tag = "1")]
    pub vote: ::std::option::Option<Vote>,
    #[prost(string, tag = "2")]
    pub chain_id: String,
}

// Copied from tendermint_proto::privval::SignedVoteResponse
/// SignedVoteResponse is a response containing a signed vote or an error
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedVoteResponse {
    #[prost(message, optional, tag = "1")]
    pub vote: ::std::option::Option<Vote>,
    #[prost(message, optional, tag = "2")]
    pub error: ::std::option::Option<RemoteSignerError>,
}

// Copied from tendermint_proto::types::CanonicalVote
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalVote {
    /// type alias for byte
    #[prost(enumeration = "SignedMsgType", tag = "1")]
    pub r#type: i32,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag = "2")]
    pub height: i64,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag = "3")]
    pub round: i64,
    #[prost(message, optional, tag = "4")]
    pub block_id: ::std::option::Option<CanonicalBlockId>,
    #[prost(message, optional, tag = "5")]
    pub timestamp: ::std::option::Option<::prost_types::Timestamp>,
    #[prost(string, tag = "6")]
    pub chain_id: String,
}

impl chain::ParseId for CanonicalVote {
    fn parse_chain_id(&self) -> Result<chain::Id, Error> {
        self.chain_id.parse()
    }
}

impl block::ParseHeight for CanonicalVote {
    fn parse_block_height(&self) -> Result<block::Height, Error> {
        block::Height::try_from(self.height)
    }
}

impl CanonicalVote {
    pub fn new(vote: Vote, chain_id: &str) -> CanonicalVote {
        CanonicalVote {
            r#type: vote.r#type,
            chain_id: chain_id.to_string(),
            block_id: match vote.block_id {
                Some(bid) => Some(CanonicalBlockId {
                    hash: bid.hash,
                    part_set_header: match bid.part_set_header {
                        Some(psh) => Some(CanonicalPartSetHeader {
                            hash: psh.hash,
                            total: psh.total,
                        }),
                        None => None,
                    },
                }),
                None => None,
            },
            height: vote.height,
            round: vote.round as i64,
            timestamp: match vote.timestamp {
                None => Some(Timestamp {
                    seconds: -62_135_596_800,
                    nanos: 0,
                }),
                Some(t) => Some(t),
            },
        }
    }
}

impl SignableMsg for SignVoteRequest {
    fn sign_bytes<B>(&self, chain_id: chain::Id, sign_bytes: &mut B) -> Result<bool, EncodeError>
    where
        B: BufMut,
    {
        let mut svr = self.clone();
        if let Some(ref mut vo) = svr.vote {
            vo.signature = vec![];
        }
        let vote = svr.vote.unwrap();
        let cv = CanonicalVote::new(vote, chain_id.as_str());

        cv.encode_length_delimited(sign_bytes)?;

        Ok(true)
    }
    fn set_signature(&mut self, sig: &ed25519::Signature) {
        if let Some(ref mut vt) = self.vote {
            vt.signature = sig.as_ref().to_vec();
        }
    }
    fn validate(&self) -> Result<(), validate::Error> {
        match self.vote {
            Some(ref v) => v.validate_basic(),
            None => Err(MissingConsensusMessage.into()),
        }
    }
    fn consensus_state(&self) -> Option<consensus::State> {
        match self.vote {
            Some(ref v) => Some(consensus::State {
                height: match block::Height::try_from(v.height) {
                    Ok(h) => h,
                    Err(_err) => return None, // TODO(tarcieri): return an error?
                },
                round: v.round as i64,
                step: 6,
                block_id: {
                    match v.block_id {
                        Some(ref b) => match b.parse_block_id() {
                            Ok(id) => Some(id),
                            Err(_) => None,
                        },
                        None => None,
                    }
                },
            }),
            None => None,
        }
    }
    fn height(&self) -> Option<i64> {
        self.vote.as_ref().map(|vote| vote.height)
    }
    fn msg_type(&self) -> Option<SignedMsgType> {
        self.vote.as_ref().and_then(|vote| vote.msg_type())
    }
}

impl ConsensusMessage for Vote {
    fn validate_basic(&self) -> Result<(), validate::Error> {
        if self.msg_type().is_none() {
            return Err(InvalidMessageType.into());
        }
        if self.height < 0 {
            return Err(NegativeHeight.into());
        }
        if self.round < 0 {
            return Err(NegativeRound.into());
        }
        if self.validator_index < 0 {
            return Err(NegativeValidatorIndex.into());
        }
        if self.validator_address.len() != VALIDATOR_ADDR_SIZE {
            return Err(InvalidValidatorAddressSize.into());
        }

        self.block_id
            .as_ref()
            .map_or(Ok(()), ConsensusMessage::validate_basic)

        // signature will be missing as the KMS provides it
    }
}

#[cfg(test)]
mod tests {
    use super::super::PartSetHeader;
    use super::*;
    use crate::amino_types::message::AminoMessage;
    use crate::amino_types::SignedMsgType;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_vote_serialization() {
        let dt = "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap();
        let t = Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let vote = Vote {
            r#type: SignedMsgType::Prevote as i32,
            height: 12345,
            round: 2,
            timestamp: Some(t),
            block_id: Some(BlockId {
                hash: b"DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA".to_vec(),
                part_set_header: Some(PartSetHeader {
                    total: 1_000_000,
                    hash: b"0022446688AACCEE1133557799BBDDFF".to_vec(),
                }),
            }),
            validator_address: vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ],
            validator_index: 56789,
            signature: vec![],
            /* signature: vec![130u8, 246, 183, 50, 153, 248, 28, 57, 51, 142, 55, 217, 194, 24,
             * 134, 212, 233, 100, 211, 10, 24, 174, 179, 117, 41, 65, 141, 134, 149, 239, 65,
             * 174, 217, 42, 6, 184, 112, 17, 7, 97, 255, 221, 252, 16, 60, 144, 30, 212, 167,
             * 39, 67, 35, 118, 192, 133, 130, 193, 115, 32, 206, 152, 91, 173, 10], */
        };
        let mut got = vec![];

        // Simulating Go's ProposalSignBytes function. Shall we make this into a function too?
        let canonical = CanonicalVote {
            r#type: vote.r#type,
            height: vote.height,
            round: vote.round as i64,
            block_id: Some(CanonicalBlockId {
                hash: vote.block_id.clone().unwrap().hash,
                part_set_header: Some(CanonicalPartSetHeader {
                    total: vote
                        .block_id
                        .clone()
                        .unwrap()
                        .part_set_header
                        .unwrap()
                        .total,
                    hash: vote.block_id.unwrap().part_set_header.unwrap().hash,
                }),
            }),
            timestamp: vote.timestamp,
            chain_id: "test_chain_id".to_string(),
        };
        canonical.encode_length_delimited(&mut got).unwrap();

        // the following vector is generated via:
        /*
           import (
               "fmt"
               prototypes "github.com/tendermint/tendermint/proto/tendermint/types"
               "github.com/tendermint/tendermint/types"
               "strings"
               "time"
           )
           func voteSerialize() {
               stamp, _ := time.Parse(time.RFC3339Nano, "2017-12-25T03:00:01.234Z")
               vote := &types.Vote{
                   Type:      prototypes.PrevoteType, // pre-vote
                   Height:    12345,
                   Round:     2,
                   Timestamp: stamp,
                   BlockID: types.BlockID{
                       Hash: []byte("DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA"),
                       PartSetHeader: types.PartSetHeader{
                           Total: 1000000,
                           Hash:  []byte("0022446688AACCEE1133557799BBDDFF"),
                       },
                   },
                   ValidatorAddress: []byte{0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21,
                       0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35}, ValidatorIndex: 56789}
               signBytes := types.VoteSignBytes("test_chain_id", vote.ToProto())
               fmt.Println(strings.Join(strings.Split(fmt.Sprintf("%v", signBytes), " "), ", "))
           }
        */

        let want = vec![
            124, 8, 1, 17, 57, 48, 0, 0, 0, 0, 0, 0, 25, 2, 0, 0, 0, 0, 0, 0, 0, 34, 74, 10, 32,
            68, 69, 65, 68, 66, 69, 69, 70, 68, 69, 65, 68, 66, 69, 69, 70, 66, 65, 70, 66, 65, 70,
            66, 65, 70, 66, 65, 70, 66, 65, 70, 65, 18, 38, 8, 192, 132, 61, 18, 32, 48, 48, 50,
            50, 52, 52, 54, 54, 56, 56, 65, 65, 67, 67, 69, 69, 49, 49, 51, 51, 53, 53, 55, 55, 57,
            57, 66, 66, 68, 68, 70, 70, 42, 11, 8, 177, 211, 129, 210, 5, 16, 128, 157, 202, 111,
            50, 13, 116, 101, 115, 116, 95, 99, 104, 97, 105, 110, 95, 105, 100,
        ];
        assert_eq!(got, want);
    }

    #[test]
    fn test_sign_bytes_compatibility() {
        let cv = CanonicalVote::new(Vote::default(), "");
        let mut got = vec![];
        // SignBytes are encoded using MarshalBinary and not MarshalBinaryBare
        cv.encode_length_delimited(&mut got).unwrap();
        let want = vec![
            0xd, 0x2a, 0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
        ];
        assert_eq!(got, want);

        // with proper (fixed size) height and round (Precommit):
        {
            let mut vt_precommit = Vote::default();
            vt_precommit.height = 1;
            vt_precommit.round = 1;
            vt_precommit.r#type = SignedMsgType::Precommit as i32; // precommit
            println!("{:?}", vt_precommit);
            let cv_precommit = CanonicalVote::new(vt_precommit, "");
            let got = AminoMessage::bytes_vec(&cv_precommit);
            let want = vec![
                0x8,  // (field_number << 3) | wire_type
                0x2,  // PrecommitType
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // round
                0x2a, // (field_number << 3) | wire_type
                // remaining fields (timestamp):
                0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
            ];
            assert_eq!(got, want);
        }
        // with proper (fixed size) height and round (Prevote):
        {
            let mut vt_prevote = Vote::default();
            vt_prevote.height = 1;
            vt_prevote.round = 1;
            vt_prevote.r#type = SignedMsgType::Prevote as i32;

            let cv_prevote = CanonicalVote::new(vt_prevote, "");

            let got = AminoMessage::bytes_vec(&cv_prevote);

            let want = vec![
                0x8,  // (field_number << 3) | wire_type
                0x1,  // PrevoteType
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // round
                0x2a, // (field_number << 3) | wire_type
                // remaining fields (timestamp):
                0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
            ];
            assert_eq!(got, want);
        }
        // with proper (fixed size) height and round (msg typ missing):
        {
            let mut vt_no_type = Vote::default();
            vt_no_type.height = 1;
            vt_no_type.round = 1;

            let cv = CanonicalVote::new(vt_no_type, "");
            let got = AminoMessage::bytes_vec(&cv);

            let want = vec![
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
                // remaining fields (timestamp):
                0x2a, 0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
            ];
            assert_eq!(got, want);
        }
        // containing non-empty chain_id:
        {
            let mut no_vote_type2 = Vote::default();
            no_vote_type2.height = 1;
            no_vote_type2.round = 1;

            let with_chain_id = CanonicalVote::new(no_vote_type2, "test_chain_id");
            got = AminoMessage::bytes_vec(&with_chain_id);
            let want = vec![
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
                // remaining fields:
                0x2a, // (field_number << 3) | wire_type
                0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff,
                0x1,  // timestamp
                0x32, // (field_number << 3) | wire_type
                0xd, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x63, 0x68, 0x61, 0x69, 0x6e, 0x5f, 0x69,
                0x64, // chainID
            ];
            assert_eq!(got, want);
        }
    }

    #[test]
    fn test_vote_rountrip_with_sig() {
        let dt = "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap();
        let t = Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let vote = Vote {
            validator_address: vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ],
            validator_index: 56789,
            height: 12345,
            round: 2,
            timestamp: Some(t),
            r#type: 0x01,
            block_id: Some(BlockId {
                hash: b"hash".to_vec(),
                part_set_header: Some(PartSetHeader {
                    total: 1_000_000,
                    hash: b"parts_hash".to_vec(),
                }),
            }),
            // signature: None,
            signature: vec![
                130u8, 246, 183, 50, 153, 248, 28, 57, 51, 142, 55, 217, 194, 24, 134, 212, 233,
                100, 211, 10, 24, 174, 179, 117, 41, 65, 141, 134, 149, 239, 65, 174, 217, 42, 6,
                184, 112, 17, 7, 97, 255, 221, 252, 16, 60, 144, 30, 212, 167, 39, 67, 35, 118,
                192, 133, 130, 193, 115, 32, 206, 152, 91, 173, 10,
            ],
        };
        let mut got = vec![];
        let _have = vote.encode(&mut got);
        let v = Vote::decode(got.as_ref()).unwrap();

        assert_eq!(v, vote);
        // SignVoteRequest
        {
            let svr = SignVoteRequest {
                vote: Some(vote),
                chain_id: "test_chain_id".to_string(),
            };
            let mut got = vec![];
            let _have = svr.encode(&mut got);

            let svr2 = SignVoteRequest::decode(got.as_ref()).unwrap();
            assert_eq!(svr, svr2);
        }
    }

    #[test]
    fn test_deserialization() {
        let encoded = vec![
            10, 122, 8, 1, 16, 185, 96, 24, 2, 34, 74, 10, 32, 68, 69, 65, 68, 66, 69, 69, 70, 68,
            69, 65, 68, 66, 69, 69, 70, 66, 65, 70, 66, 65, 70, 66, 65, 70, 66, 65, 70, 66, 65, 70,
            65, 18, 38, 8, 192, 132, 61, 18, 32, 48, 48, 50, 50, 52, 52, 54, 54, 56, 56, 65, 65,
            67, 67, 69, 69, 49, 49, 51, 51, 53, 53, 55, 55, 57, 57, 66, 66, 68, 68, 70, 70, 42, 11,
            8, 177, 211, 129, 210, 5, 16, 128, 157, 202, 111, 50, 20, 163, 178, 204, 221, 113, 134,
            241, 104, 95, 33, 242, 72, 42, 244, 251, 52, 70, 168, 75, 53, 56, 213, 187, 3, 18, 13,
            116, 101, 115, 116, 95, 99, 104, 97, 105, 110, 95, 105, 100,
        ];
        let dt = "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap();
        let t = Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let vote = Vote {
            validator_address: vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ],
            validator_index: 56789,
            height: 12345,
            round: 2,
            timestamp: Some(t),
            r#type: 0x01,
            block_id: Some(BlockId {
                hash: b"DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA".to_vec(),
                part_set_header: Some(PartSetHeader {
                    total: 1_000_000,
                    hash: b"0022446688AACCEE1133557799BBDDFF".to_vec(),
                }),
            }),
            signature: vec![],
        };
        let want = SignVoteRequest {
            vote: Some(vote),
            chain_id: "test_chain_id".to_string(),
        };
        match SignVoteRequest::decode(encoded.as_ref()) {
            Ok(have) => {
                assert_eq!(have, want);
            }
            Err(err) => panic!(err.to_string()),
        }
    }
}
