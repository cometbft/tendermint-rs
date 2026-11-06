#![no_main]

use libfuzzer_sys::fuzz_target;
use tendermint::{block::signed_header::SignedHeader, validator};
use tendermint_proto::{
    v0_38::types::{SignedHeader as RawSignedHeader, ValidatorSet as RawValidatorSet},
    Protobuf,
};

fn roundtrip<T, Raw>(data: &[u8])
where
    T: Protobuf<Raw> + PartialEq + core::fmt::Debug,
    T: TryFrom<Raw>,
    <T as TryFrom<Raw>>::Error: core::fmt::Display,
    Raw: prost::Message + Default + From<T>,
{
    if let Ok(value) = <T as Protobuf<Raw>>::decode_vec(data) {
        let encoded = <T as Protobuf<Raw>>::encode_vec(value.clone());
        let decoded = <T as Protobuf<Raw>>::decode_vec(&encoded).expect("re-decode failed");
        assert_eq!(value, decoded);
    }
}

fuzz_target!(|data: &[u8]| {
    roundtrip::<SignedHeader, RawSignedHeader>(data);
    roundtrip::<validator::Set, RawValidatorSet>(data);
});
