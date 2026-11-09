//! Writes valid untrusted light blocks to `seeds/verify_light_block/`.
//!
//! Run with `cargo run --manifest-path fuzz/Cargo.toml --example gen_seeds`.

use std::fs;

use prost::Message;
use tendermint_fuzz::light_block;
use tendermint_proto::v0_38::types::LightBlock as RawLightBlock;

fn main() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/seeds/verify_light_block");
    fs::create_dir_all(dir).unwrap();
    for height in [2, 3, 10] {
        let lb = light_block(height);
        let raw = RawLightBlock {
            signed_header: Some(lb.signed_header.into()),
            validator_set: Some(lb.validators.into()),
        };
        fs::write(format!("{dir}/height-{height}"), raw.encode_to_vec()).unwrap();
    }
}
