//! Property tests for `tendermint_proto::encoding`.

use proptest::{collection::vec, prelude::*, sample::select};
use tendermint_proto::encoding::{base64, bech32, hex, Error};

const HEX_LOWER: &[u8] = b"0123456789abcdef";
const HEX_UPPER: &[u8] = b"0123456789ABCDEF";
const BASE64_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const BECH32_CHARSET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Mostly-valid input: characters from `alphabet`, plus some that are
/// interesting for the decoders (padding, whitespace, other ASCII, non-ASCII).
fn near_alphabet(alphabet: &'static [u8], max_len: usize) -> impl Strategy<Value = Vec<u8>> {
    let byte = prop_oneof![
        8 => select(alphabet),
        2 => Just(b'='),
        1 => select(&b" \n\t\r-_.:"[..]),
        1 => any::<u8>(),
    ];
    vec(byte, 0..max_len)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    // ---- hex ----

    #[test]
    fn hex_round_trip(data in vec(any::<u8>(), 0..128)) {
        let lower = hex::encode(&data);
        prop_assert_eq!(lower.len(), data.len() * 2);
        prop_assert!(lower.iter().all(|c| HEX_LOWER.contains(c)));
        prop_assert_eq!(hex::decode(&lower).unwrap(), data.clone());

        let upper = hex::encode_upper(&data);
        prop_assert_eq!(&upper, &lower.to_ascii_uppercase());
        prop_assert_eq!(hex::decode_upper(&upper).unwrap(), data);
    }

    /// Decoding arbitrary input never panics, and whatever is accepted is the
    /// canonical encoding of the result.
    #[test]
    fn hex_decode_is_canonical(
        lower in near_alphabet(HEX_LOWER, 64),
        upper in near_alphabet(HEX_UPPER, 64),
    ) {
        if let Ok(data) = hex::decode(&lower) {
            prop_assert_eq!(hex::encode(data), lower);
        }
        if let Ok(data) = hex::decode_upper(&upper) {
            prop_assert_eq!(hex::encode_upper(data), upper);
        }
    }

    #[test]
    fn hex_decode_to_slice_agrees_with_decode(
        src in near_alphabet(HEX_UPPER, 64),
        dst_len in 0usize..40,
    ) {
        let mut dst = vec![0u8; dst_len];
        match hex::decode_upper_to_slice(&src, &mut dst) {
            Ok(n) => {
                prop_assert_eq!(n, src.len() / 2);
                prop_assert_eq!(&dst[..n], &hex::decode_upper(&src).unwrap()[..]);
            },
            Err(Error::LengthInvalid) => {
                prop_assert!(src.len() % 2 != 0 || src.len() / 2 > dst_len);
            },
            Err(_) => prop_assert!(hex::decode_upper(&src).is_err()),
        }
    }

    // ---- base64 ----

    #[test]
    fn base64_round_trip(data in vec(any::<u8>(), 0..128)) {
        let encoded = base64::encode(&data);
        prop_assert_eq!(encoded.len(), data.len().div_ceil(3) * 4);

        let unpadded = encoded.iter().position(|&c| c == b'=').unwrap_or(encoded.len());
        prop_assert!(encoded[..unpadded].iter().all(|c| BASE64_ALPHABET.contains(c)));
        prop_assert!(encoded[unpadded..].iter().all(|&c| c == b'='));
        prop_assert!(encoded.len() - unpadded <= 2);

        prop_assert_eq!(base64::decode(&encoded).unwrap(), data.clone());
        // Missing padding is tolerated
        prop_assert_eq!(base64::decode(&encoded[..unpadded]).unwrap(), data);
    }

    /// Decoding arbitrary input never panics, and whatever is accepted decodes
    /// to the number of bytes implied by its non-padding characters.
    #[test]
    fn base64_decode_never_panics(src in near_alphabet(BASE64_ALPHABET, 64)) {
        if let Ok(data) = base64::decode(&src) {
            let data_chars = src.iter().filter(|&&c| c != b'=').count();
            prop_assert_eq!(data.len(), data_chars * 3 / 4);
            prop_assert!(src.iter().all(|c| *c == b'=' || BASE64_ALPHABET.contains(c)));
            // Re-encoding gives the same data part, up to the unused low bits
            // of the last character.
            let reencoded = base64::encode(&data);
            let common = data.len() * 4 / 3;
            prop_assert_eq!(&reencoded[..common], &src[..common]);
        }
    }

    #[test]
    fn base64_decode_to_slice_agrees_with_decode(
        src in near_alphabet(BASE64_ALPHABET, 64),
        dst_len in 0usize..56,
    ) {
        let mut dst = vec![0u8; dst_len];
        let decoded = base64::decode(&src);
        match base64::decode_to_slice(&src, &mut dst) {
            Ok(n) => prop_assert_eq!(&dst[..n], &decoded.unwrap()[..]),
            Err(Error::LengthInvalid) => {
                prop_assert!(decoded.map_or(true, |d| d.len() > dst_len));
            },
            Err(e) => prop_assert_eq!(decoded, Err(e)),
        }
    }

    // ---- bech32 ----

    #[test]
    fn bech32_encode_is_valid(
        hrp in vec(33u8..=126, 1..20),
        data in vec(any::<u8>(), 0..64),
    ) {
        let hrp = String::from_utf8(hrp).unwrap();
        let encoded = bech32::encode(&hrp, &data);

        let (encoded_hrp, rest) = encoded.split_at(hrp.len());
        prop_assert_eq!(encoded_hrp, hrp.as_str());
        prop_assert!(rest.starts_with('1'));

        let values: Vec<u8> = rest[1..]
            .bytes()
            .map(|c| BECH32_CHARSET.iter().position(|&x| x == c).unwrap() as u8)
            .collect();
        prop_assert_eq!(values.len(), (data.len() * 8).div_ceil(5) + 6);
        prop_assert!(bech32_checksum_is_valid(hrp.as_bytes(), &values));
        prop_assert_eq!(from_base32(&values[..values.len() - 6]), data);
    }
}

/// BIP-173 checksum verification
fn bech32_checksum_is_valid(hrp: &[u8], values: &[u8]) -> bool {
    const GENERATOR: [u32; 5] = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
    let expanded = hrp
        .iter()
        .map(|b| b >> 5)
        .chain([0])
        .chain(hrp.iter().map(|b| b & 0x1f))
        .chain(values.iter().copied());

    let mut chk: u32 = 1;
    for v in expanded {
        let top = chk >> 25;
        chk = ((chk & 0x1ff_ffff) << 5) ^ u32::from(v);
        for (i, g) in GENERATOR.iter().enumerate() {
            if (top >> i) & 1 == 1 {
                chk ^= g;
            }
        }
    }
    chk == 1
}

/// Regroup 5-bit values into bytes, dropping the zero padding bits.
fn from_base32(values: &[u8]) -> Vec<u8> {
    let mut acc: u32 = 0;
    let mut bits = 0;
    let mut out = Vec::new();
    for v in values {
        acc = (acc << 5) | u32::from(*v);
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    assert_eq!(acc & ((1 << bits) - 1), 0, "non-zero padding bits");
    out
}
