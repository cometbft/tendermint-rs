//! Bech32 ([BIP-173]) encoding.
//!
//! Only encoding is supported. Unlike the hex and Base64 modules, this is not
//! constant time; it is meant for public data such as public keys and addresses.
//!
//! [BIP-173]: https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki

use zeroize::Zeroize;

use crate::prelude::*;

/// Separator between the human-readable part and the data part
const SEPARATOR: char = '1';

/// Bech32 alphabet (lower case)
const CHARSET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Number of 5-bit characters in the checksum
const CHECKSUM_LEN: usize = 6;

/// Encode `data` as lower-case Bech32 with the human-readable part `hrp`.
///
/// `hrp` is copied into the output as is.
pub fn encode<S, D>(hrp: S, data: D) -> String
where
    S: AsRef<str>,
    D: AsRef<[u8]>,
{
    let hrp = hrp.as_ref();
    let mut base32_data = to_base32(data.as_ref());
    let checksum = checksum(hrp.as_bytes(), &base32_data);

    let mut result = String::with_capacity(hrp.len() + 1 + base32_data.len() + CHECKSUM_LEN);
    result.push_str(hrp);
    result.push(SEPARATOR);
    for value in base32_data.iter().chain(checksum.iter()) {
        result.push(char::from(CHARSET[usize::from(*value)]));
    }

    base32_data.zeroize();
    result
}

/// Regroup 8-bit bytes into 5-bit values, zero-padding the last one.
fn to_base32(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity((data.len() * 8).div_ceil(5));
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;

    for byte in data {
        acc = (acc << 8) | u32::from(*byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            result.push(((acc >> bits) & 0x1f) as u8);
        }
    }

    if bits > 0 {
        result.push(((acc << (5 - bits)) & 0x1f) as u8);
    }

    acc.zeroize();
    result
}

/// Compute the BIP-173 checksum over the human-readable part and the 5-bit data.
fn checksum(hrp: &[u8], data: &[u8]) -> [u8; CHECKSUM_LEN] {
    let mut p = Polymod::default();
    for b in hrp {
        p.input(b >> 5);
    }
    p.input(0);
    for b in hrp {
        p.input(b & 0x1f);
    }
    for b in data {
        p.input(*b);
    }
    for _ in 0..CHECKSUM_LEN {
        p.input(0);
    }

    let value = p.0 ^ 1;
    let mut checksum = [0u8; CHECKSUM_LEN];
    for (i, c) in checksum.iter_mut().enumerate() {
        *c = ((value >> (5 * (CHECKSUM_LEN - 1 - i))) & 0x1f) as u8;
    }
    checksum
}

/// BCH code checksum state
struct Polymod(u32);

impl Default for Polymod {
    fn default() -> Self {
        Polymod(1)
    }
}

impl Polymod {
    const GENERATOR: [u32; 5] = [
        0x3b6a_57b2,
        0x2650_8e6d,
        0x1ea1_19fa,
        0x3d42_33dd,
        0x2a14_62b3,
    ];

    fn input(&mut self, value: u8) {
        let top = self.0 >> 25;
        self.0 = ((self.0 & 0x1ff_ffff) << 5) ^ u32::from(value);
        for (i, g) in Self::GENERATOR.iter().enumerate() {
            if (top >> i) & 1 == 1 {
                self.0 ^= g;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Valid BIP-173 test vectors: (hrp, data, encoding)
    const VECTORS: &[(&str, &[u8], &str)] = &[
        ("a", &[], "a12uel5l"),
        (
            "an83characterlonghumanreadablepartthatcontainsthenumber1andtheexcludedcharactersbio",
            &[],
            "an83characterlonghumanreadablepartthatcontainsthenumber1andtheexcludedcharactersbio1tt5tgs",
        ),
        (
            "abcdef",
            &[
                0, 68, 50, 20, 199, 66, 84, 182, 53, 207, 132, 101, 58, 86, 215, 198, 117, 190, 119,
                223,
            ],
            "abcdef1qpzry9x8gf2tvdw0s3jn54khce6mua7lmqqqxw",
        ),
        (
            "1",
            &[0; 51],
            "11qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqc8247j",
        ),
        (
            "split",
            &[
                197, 243, 139, 112, 48, 95, 81, 155, 246, 109, 133, 251, 108, 240, 48, 88, 243, 221,
                228, 99, 236, 215, 145, 143, 45, 199, 67, 145, 143, 45,
            ],
            "split1checkupstagehandshakeupstreamerranterredcaperred2y9e3w",
        ),
        ("?", &[], "?1ezyfcl"),
    ];

    #[test]
    fn encode_test_vectors() {
        for (hrp, data, encoded) in VECTORS {
            assert_eq!(encode(hrp, data), *encoded);
        }
    }

    #[test]
    fn to_base32_pads_last_group() {
        assert_eq!(
            to_base32(&[1, 2, 3, 5, 9, 17, 33, 65, 129]),
            [0, 4, 1, 0, 6, 1, 8, 9, 2, 4, 16, 20, 3, 0, 8]
        );
    }
}
