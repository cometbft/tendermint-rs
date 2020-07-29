//! Helper functions

use serde::de::DeserializeOwned;
use signatory::signature::Verifier;
use signatory_dalek::Ed25519Verifier;
use simple_error::*;
use std::io::{self, Read};
use tendermint::{amino_types, signature::Signature, vote};

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

pub fn get_vote_sign_bytes(chain_id: &str, vote: &vote::Vote) -> Vec<u8> {
    let signed_vote = vote::SignedVote::new(
        amino_types::vote::Vote::from(vote),
        chain_id,
        vote.validator_address,
        vote.signature.clone(),
    );
    signed_vote.sign_bytes()
}

pub fn verify_signature(verifier: &Ed25519Verifier, msg: &[u8], signature: &Signature) -> bool {
    match signature {
        tendermint::signature::Signature::Ed25519(sig) => verifier.verify(msg, sig).is_ok(),
    }
}
