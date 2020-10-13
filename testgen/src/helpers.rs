//! Helper functions

use serde::de::DeserializeOwned;
use simple_error::*;
use std::io::{self, Read};
use tendermint::{
    chain, public_key,
    signature::{Signature, Verifier},
    vote, Time,
};

/// A macro that generates a complete setter method from a one-liner with necessary information
#[macro_export]
macro_rules! set_option {
    ($name:ident, $t:ty) => {
        pub fn $name(mut self, $name: $t) -> Self {
            self.$name = Some($name.clone());
            self
        }
    };
    ($name:ident, $t:ty, $val:expr) => {
        pub fn $name(mut self, $name: $t) -> Self {
            self.$name = $val;
            self
        }
    };
}

/// Tries to parse a string as the given type; otherwise returns the input wrapped in SimpleError
pub fn parse_as<T: DeserializeOwned>(input: &str) -> Result<T, SimpleError> {
    match serde_json::from_str(input) {
        Ok(res) => Ok(res),
        Err(e) => Err(SimpleError::new(e.to_string())),
    }
}

pub fn read_stdin() -> Result<String, SimpleError> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(e) => Err(SimpleError::new(e.to_string())),
    }
}

pub fn get_vote_sign_bytes(chain_id: chain::Id, vote: &vote::Vote) -> Vec<u8> {
    let signed_vote = vote::SignedVote::new(
        vote.clone(),
        chain_id,
        vote.validator_address,
        vote.signature,
    );
    signed_vote.sign_bytes()
}

pub fn verify_signature(verifier: &public_key::Ed25519, msg: &[u8], signature: &Signature) -> bool {
    match signature {
        tendermint::signature::Signature::Ed25519(sig) => verifier.verify(msg, sig).is_ok(),
        _ => false,
    }
}

pub fn get_time(abs: u64) -> Time {
    (std::time::UNIX_EPOCH + std::time::Duration::from_secs(abs)).into()
}
