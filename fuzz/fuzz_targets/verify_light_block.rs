#![no_main]

use std::{sync::LazyLock, time::Duration};

use libfuzzer_sys::fuzz_target;
use prost::Message;
use tendermint::{block::signed_header::SignedHeader, validator};
use tendermint_fuzz::{light_block, now};
use tendermint_light_client_verifier::{
    options::Options,
    types::{LightBlock, UntrustedBlockState},
    ProdVerifier, Verifier,
};
use tendermint_proto::v0_38::types::LightBlock as RawLightBlock;

static TRUSTED: LazyLock<LightBlock> = LazyLock::new(|| light_block(1));

fuzz_target!(|data: &[u8]| {
    let Ok(raw) = RawLightBlock::decode(data) else {
        return;
    };
    let (Some(raw_header), Some(raw_validators)) = (raw.signed_header, raw.validator_set) else {
        return;
    };
    let (Ok(signed_header), Ok(validators)) = (
        SignedHeader::try_from(raw_header),
        validator::Set::try_from(raw_validators),
    ) else {
        return;
    };

    let untrusted = || UntrustedBlockState {
        signed_header: &signed_header,
        validators: &validators,
        next_validators: None,
    };
    let options = Options {
        trust_threshold: Default::default(),
        trusting_period: Duration::from_secs(3600),
        clock_drift: Duration::from_secs(5),
    };
    let verifier = ProdVerifier::default();
    let _ = verifier.verify_update_header(untrusted(), TRUSTED.as_trusted_state(), &options, now());
    let _ = verifier.verify_misbehaviour_header(
        untrusted(),
        TRUSTED.as_trusted_state(),
        &options,
        now(),
    );
});
