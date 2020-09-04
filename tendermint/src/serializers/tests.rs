//! Serialization tests

use crate::account::Id;
use crate::block::CommitSig;
use crate::test::test_serialization_roundtrip;
use crate::time::Time;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const EXAMPLE_SIGNATURE: [u8; 64] = [
    63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46, 45, 44, 43, 42, 41, 40,
    39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16,
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
];

#[test]
fn serde_integer_string() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct IntegerTests {
        #[serde(with = "super::from_str")]
        unsigned: u64,

        #[serde(with = "super::from_str")]
        signed: i64,
    }

    test_serialization_roundtrip::<IntegerTests>(
        r#"
{
  "unsigned": "9007199254740992",
  "signed": "-9007199254740994"
}
"#,
    );
}

#[test]
fn serde_duration_string() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct DurationTests {
        #[serde(with = "super::time_duration")]
        duration: Duration,
    }

    test_serialization_roundtrip::<DurationTests>(
        r#"
{
  "duration": "15000000001"
}
"#,
    );
}

#[test]
fn serde_vec_string() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct BytesTests {
        #[serde(with = "super::bytes::hexstring")]
        myhexbytes: Vec<u8>,

        #[serde(with = "super::bytes::base64string")]
        mybase64bytes: Vec<u8>,

        #[serde(with = "super::bytes::string")]
        stringifiedbytes: Vec<u8>,
    }

    test_serialization_roundtrip::<BytesTests>(
        r#"
{
  "myhexbytes": "412042FF00",
  "mybase64bytes": "TXlTdHJpbmcgZGVjb2RlZC4=",
  "stringifiedbytes": "hello"
}
"#,
    );

    test_serialization_roundtrip::<BytesTests>(
        r#"
{
  "myhexbytes": null,
  "mybase64bytes": null,
  "stringifiedbytes": null
}
"#,
    );
}

// Todo: https://github.com/informalsystems/tendermint-rs/issues/260 - CommitSig validator address missing in Absent vote
#[test]
fn deserialize_commit_sig_absent_vote() {
    let incoming = r#"
    {
        "block_id_flag": 1,
        "timestamp": "0001-01-01T00:00:00Z"
    }
    "#;

    let result: CommitSig = serde_json::from_str(&incoming).unwrap();

    if let CommitSig::BlockIDFlagAbsent = result {
    } else {
        panic!(format!("expected BlockIDFlagAbsent, received {:?}", result));
    }
}

#[test]
fn deserialize_commit_sig_commit_vote() {
    let incoming = r#"
    {
        "block_id_flag": 2,
        "validator_address": "4142434445464748494A4B4C4D4E4F5051525354",
        "timestamp": "1970-01-01T00:00:00Z",
        "signature": "Pz49PDs6OTg3NjU0MzIxMC8uLSwrKikoJyYlJCMiISAfHh0cGxoZGBcWFRQTEhEQDw4NDAsKCQgHBgUEAwIBAA=="
    }
    "#;

    let result: CommitSig = serde_json::from_str(&incoming).unwrap();

    if let CommitSig::BlockIDFlagCommit {
        validator_address,
        timestamp,
        signature,
    } = result
    {
        assert_eq!(
            validator_address,
            Id::new([
                65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84
            ])
        );
        assert_eq!(timestamp, Time::unix_epoch());
        assert_eq!(
            signature,
            crate::signature::Signature::Ed25519(ed25519::Signature::new(EXAMPLE_SIGNATURE))
        );
    } else {
        panic!(format!("expected BlockIDFlagCommit, received {:?}", result));
    }
}

#[test]
fn deserialize_commit_sig_nil_vote() {
    let incoming = r#"
    {
        "block_id_flag": 3,
        "validator_address": "4142434445464748494A4B4C4D4E4F5051525354",
        "timestamp": "1970-01-01T00:00:00Z",
        "signature": "Pz49PDs6OTg3NjU0MzIxMC8uLSwrKikoJyYlJCMiISAfHh0cGxoZGBcWFRQTEhEQDw4NDAsKCQgHBgUEAwIBAA=="
    }
    "#;

    let result: CommitSig = serde_json::from_str(&incoming).unwrap();

    if let CommitSig::BlockIDFlagNil {
        validator_address,
        timestamp,
        signature,
    } = result
    {
        assert_eq!(
            validator_address,
            Id::new([
                65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84
            ])
        );
        assert_eq!(timestamp, Time::unix_epoch());
        assert_eq!(
            signature,
            crate::signature::Signature::Ed25519(ed25519::Signature::new(EXAMPLE_SIGNATURE))
        );
    } else {
        panic!(format!("expected BlockIDFlagNil, received {:?}", result));
    }
}
