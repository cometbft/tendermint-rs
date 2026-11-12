//! Light-client commit verification for a validator set containing an
//! ML-DSA-65 validator, using a block produced and checked by CometBFT v0.40.0.
#![cfg(feature = "mldsa65")]

use serde_json::Value;
use tendermint::{block::signed_header::SignedHeader, validator};
use tendermint_light_client_verifier::{types::UntrustedBlockState, ProdVerifier, Verdict};

const FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../tendermint/tests/support/mldsa65/fixture.json"
));

fn load() -> (SignedHeader, validator::Set) {
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let light_block = &fixture["light_block"];
    (
        serde_json::from_value(light_block["signed_header"].clone()).unwrap(),
        serde_json::from_value(light_block["validator_set"].clone()).unwrap(),
    )
}

fn verify(signed_header: &SignedHeader, validators: &validator::Set) -> Verdict {
    ProdVerifier::default().verify_commit(&UntrustedBlockState {
        signed_header,
        validators,
        next_validators: None,
    })
}

#[test]
fn commit_with_mldsa65_validator_verifies() {
    let (signed_header, validators) = load();
    assert_eq!(verify(&signed_header, &validators), Verdict::Success);
}

#[test]
fn tampered_mldsa65_commit_signature_is_rejected() {
    let (mut signed_header, validators) = load();
    let mldsa_address = validators
        .validators()
        .iter()
        .find(|v| matches!(v.pub_key, tendermint::PublicKey::MlDsa65(_)))
        .unwrap()
        .address;

    let mut tampered = false;
    for sig in signed_header.commit.signatures.iter_mut() {
        if let tendermint::block::CommitSig::BlockIdFlagCommit {
            validator_address,
            signature: Some(signature),
            ..
        } = sig
        {
            if *validator_address == mldsa_address {
                let mut bytes = signature.clone().into_bytes();
                bytes[0] ^= 0x01;
                *signature = tendermint::Signature::try_from(bytes).unwrap();
                tampered = true;
            }
        }
    }
    assert!(tampered);
    assert_ne!(verify(&signed_header, &validators), Verdict::Success);
}
