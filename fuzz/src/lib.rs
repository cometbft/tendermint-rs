//! Fixtures shared by the light-client fuzz target and its seed generator.

use std::time::Duration;

use tendermint::Time;
use tendermint_light_client_verifier::types::LightBlock;
use tendermint_testgen::{light_block::LightBlock as TestgenLightBlock, Generator};

pub const CHAIN_ID: &str = "fuzz-chain";

pub fn genesis_time() -> Time {
    Time::from_unix_timestamp(1_700_000_000, 0).unwrap()
}

pub fn now() -> Time {
    (genesis_time() + Duration::from_secs(60)).unwrap()
}

/// A block signed by the default testgen validators at `height`, `height` seconds after genesis.
pub fn light_block(height: u64) -> LightBlock {
    let time = (genesis_time() + Duration::from_secs(height)).unwrap();
    let lb =
        TestgenLightBlock::new_default_with_time_and_chain_id(CHAIN_ID.to_owned(), time, height)
            .generate()
            .unwrap();
    LightBlock {
        signed_header: lb.signed_header,
        validators: lb.validators,
        next_validators: lb.next_validators,
        provider: lb.provider,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tendermint_light_client_verifier::{options::Options, ProdVerifier, Verdict, Verifier};

    use super::*;

    #[test]
    fn seed_blocks_verify_against_trusted_block() {
        let trusted = light_block(1);
        let options = Options {
            trust_threshold: Default::default(),
            trusting_period: Duration::from_secs(3600),
            clock_drift: Duration::from_secs(5),
        };
        for height in [2, 3, 10] {
            let untrusted = light_block(height);
            let verdict = ProdVerifier::default().verify_update_header(
                untrusted.as_untrusted_state(),
                trusted.as_trusted_state(),
                &options,
                now(),
            );
            assert_eq!(verdict, Verdict::Success, "height {height}");
        }
    }
}
