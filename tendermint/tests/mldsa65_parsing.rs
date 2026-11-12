//! Parsing behaviour for chains with ML-DSA-65 validators that does not depend
//! on the `mldsa65` feature: key-type names in consensus params and the
//! ML-DSA-65 signature length.

use serde_json::Value;
use subtle_encoding::hex;
use tendermint::{consensus::params::ValidatorParams, public_key::Algorithm, Signature};
use tendermint_proto::v0_38::types::ValidatorParams as RawValidatorParams;

const FIXTURE: &str = include_str!("support/mldsa65/fixture.json");

#[test]
fn algorithm_json_roundtrip() {
    let alg: Algorithm = serde_json::from_str("\"ml_dsa_65\"").unwrap();
    assert_eq!(alg, Algorithm::MlDsa65);
    assert_eq!(serde_json::to_string(&alg).unwrap(), "\"ml_dsa_65\"");

    let params: ValidatorParams =
        serde_json::from_str(r#"{"pub_key_types":["ed25519","ml_dsa_65"]}"#).unwrap();
    assert_eq!(
        params.pub_key_types,
        vec![Algorithm::Ed25519, Algorithm::MlDsa65]
    );
}

#[test]
fn validator_params_proto_roundtrip() {
    let raw = RawValidatorParams {
        pub_key_types: vec!["ml_dsa_65".to_string(), "ed25519".to_string()],
    };
    let params = ValidatorParams::try_from(raw.clone()).unwrap();
    assert_eq!(
        params.pub_key_types,
        vec![Algorithm::MlDsa65, Algorithm::Ed25519]
    );
    assert_eq!(RawValidatorParams::from(params), raw);
}

#[test]
fn mldsa65_signature_length_is_accepted() {
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let sig = hex::decode(fixture["key"]["sig_hex"].as_str().unwrap()).unwrap();
    assert_eq!(sig.len(), tendermint::signature::MLDSA65_SIGNATURE_LENGTH);
    assert!(Signature::try_from(sig.as_slice()).is_ok());
    assert!(Signature::try_from(&sig[..sig.len() - 1]).is_err());
}

/// Without the feature, an ML-DSA-65 validator set is rejected with an error
/// rather than a panic or a silently wrong key.
#[cfg(not(feature = "mldsa65"))]
#[test]
fn mldsa65_validator_set_fails_cleanly_without_feature() {
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let set = serde_json::from_value::<tendermint::validator::Set>(
        fixture["light_block"]["validator_set"].clone(),
    );
    assert!(set.is_err());
}
