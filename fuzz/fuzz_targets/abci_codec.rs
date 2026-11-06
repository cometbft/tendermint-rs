#![no_main]

use bytes::BytesMut;
use libfuzzer_sys::fuzz_target;
use tendermint_abci::codec::decode_length_delimited;
use tendermint_proto::v0_38::abci::{Request, Response};

fn drain<M: prost::Message + Default>(data: &[u8]) {
    let mut buf = BytesMut::from(data);
    while let Ok(Some(_)) = decode_length_delimited::<M>(&mut buf) {}
}

fuzz_target!(|data: &[u8]| {
    drain::<Request>(data);
    drain::<Response>(data);
});
