//! Hex encoding and decoding in (almost) data-independent constant time.

use super::{ends_with_whitespace, Error};
use crate::prelude::*;

/// Encode `bytes` as lower-case hexadecimal.
pub fn encode<B: AsRef<[u8]>>(bytes: B) -> Vec<u8> {
    encode_with_case(bytes.as_ref(), Case::Lower)
}

/// Encode `bytes` as upper-case hexadecimal.
pub fn encode_upper<B: AsRef<[u8]>>(bytes: B) -> Vec<u8> {
    encode_with_case(bytes.as_ref(), Case::Upper)
}

/// Decode lower-case hexadecimal.
pub fn decode<B: AsRef<[u8]>>(encoded: B) -> Result<Vec<u8>, Error> {
    decode_with_case(encoded.as_ref(), Case::Lower)
}

/// Decode upper-case hexadecimal.
pub fn decode_upper<B: AsRef<[u8]>>(encoded: B) -> Result<Vec<u8>, Error> {
    decode_with_case(encoded.as_ref(), Case::Upper)
}

/// Decode upper-case hexadecimal from `src` into the start of `dst`, returning
/// the number of bytes written.
///
/// Fails with [`Error::LengthInvalid`] if `src` has odd length or `dst` is too short.
pub fn decode_upper_to_slice(src: &[u8], dst: &mut [u8]) -> Result<usize, Error> {
    decode_to_slice(src, dst, Case::Upper)
}

fn encode_with_case(src: &[u8], case: Case) -> Vec<u8> {
    let mut dst = vec![0u8; src.len() * 2];
    for (src_byte, dst_pair) in src.iter().zip(dst.chunks_exact_mut(2)) {
        dst_pair[0] = case.encode_nibble(src_byte >> 4);
        dst_pair[1] = case.encode_nibble(src_byte & 0x0f);
    }
    dst
}

fn decode_with_case(src: &[u8], case: Case) -> Result<Vec<u8>, Error> {
    // Checked before trailing whitespace, unlike in `decode_to_slice`, to keep
    // the error precedence of subtle-encoding.
    if src.len() % 2 != 0 {
        return Err(Error::LengthInvalid);
    }

    let mut dst = vec![0u8; src.len() / 2];
    let len = decode_to_slice(src, &mut dst, case)?;
    debug_assert_eq!(len, dst.len());
    Ok(dst)
}

fn decode_to_slice(src: &[u8], dst: &mut [u8], case: Case) -> Result<usize, Error> {
    if ends_with_whitespace(src) {
        return Err(Error::TrailingWhitespace);
    }

    if src.len() % 2 != 0 {
        return Err(Error::LengthInvalid);
    }

    let dst_len = src.len() / 2;
    let dst = dst.get_mut(..dst_len).ok_or(Error::LengthInvalid)?;

    let mut err: usize = 0;
    for (dst_byte, src_pair) in dst.iter_mut().zip(src.chunks_exact(2)) {
        let byte = (case.decode_nibble(src_pair[0]) << 4) | case.decode_nibble(src_pair[1]);
        err |= byte >> 8;
        *dst_byte = byte as u8;
    }

    if err == 0 {
        Ok(dst_len)
    } else {
        Err(Error::EncodingInvalid)
    }
}

/// Lower (`0-9a-f`) or upper (`0-9A-F`) case hex alphabet
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Case {
    Lower,
    Upper,
}

impl Case {
    /// Decode a single hex character to its 4-bit value. Invalid characters
    /// decode to a value with bits set above the low nibble.
    #[inline]
    fn decode_nibble(self, src: u8) -> usize {
        let byte = src as isize;
        let mut ret: isize = -1;

        // 0-9  0x30-0x39
        // if (byte > 0x2f && byte < 0x3a) ret += byte - 0x30 + 1; // -47
        ret += (((0x2fisize - byte) & (byte - 0x3a)) >> 8) & (byte - 47);

        ret += match self {
            Case::Lower => {
                // a-f  0x61-0x66
                // if (byte > 0x60 && byte < 0x67) ret += byte - 0x61 + 10 + 1; // -86
                (((0x60isize - byte) & (byte - 0x67)) >> 8) & (byte - 86)
            },
            Case::Upper => {
                // A-F  0x41-0x46
                // if (byte > 0x40 && byte < 0x47) ret += byte - 0x41 + 10 + 1; // -54
                (((0x40isize - byte) & (byte - 0x47)) >> 8) & (byte - 54)
            },
        };

        ret as usize
    }

    /// Encode a 4-bit value as a single hex character.
    #[inline]
    fn encode_nibble(self, src: u8) -> u8 {
        let mut ret = src as isize + 0x30;

        ret += match self {
            // a-f  0x61-0x66
            Case::Lower => ((0x39isize - ret) >> 8) & (0x61isize - 0x3a),
            // A-F  0x41-0x46
            Case::Upper => ((0x39isize - ret) >> 8) & (0x41isize - 0x3a),
        };

        ret as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (raw, lower-case hex) test vectors
    const VECTORS: &[(&[u8], &[u8])] = &[
        (b"", b""),
        (b"\0", b"00"),
        (b"***", b"2a2a2a"),
        (b"\x01\x02\x03\x04", b"01020304"),
        (b"\xAD\xAD\xAD\xAD\xAD", b"adadadadad"),
        (b"\xFF\xFF\xFF\xFF\xFF", b"ffffffffff"),
    ];

    #[test]
    fn encode_test_vectors() {
        for (raw, hex) in VECTORS {
            assert_eq!(encode(raw), *hex);
            assert_eq!(encode_upper(raw), hex.to_ascii_uppercase());
        }
    }

    #[test]
    fn decode_test_vectors() {
        for (raw, hex) in VECTORS {
            assert_eq!(decode(hex).unwrap(), *raw);
            assert_eq!(decode_upper(hex.to_ascii_uppercase()).unwrap(), *raw);
        }
    }

    #[test]
    fn all_bytes_round_trip() {
        let data: Vec<u8> = (0..=255).collect();
        assert_eq!(decode(encode(&data)).unwrap(), data);
        assert_eq!(decode_upper(encode_upper(&data)).unwrap(), data);
    }

    #[test]
    fn case_is_enforced() {
        assert_eq!(decode("0A"), Err(Error::EncodingInvalid));
        assert_eq!(decode_upper("0a"), Err(Error::EncodingInvalid));
    }

    #[test]
    fn rejects_invalid_input() {
        assert_eq!(decode("12345"), Err(Error::LengthInvalid));
        assert_eq!(decode("zz"), Err(Error::EncodingInvalid));
        assert_eq!(decode("0g"), Err(Error::EncodingInvalid));
        assert_eq!(decode("00 \n"), Err(Error::TrailingWhitespace));
        // `decode` checks for odd length before trailing whitespace,
        // `decode_upper_to_slice` the other way around (as in subtle-encoding).
        assert_eq!(decode("00\n"), Err(Error::LengthInvalid));
        assert_eq!(
            decode_upper_to_slice(b"00 ", &mut [0u8; 2]),
            Err(Error::TrailingWhitespace)
        );
    }

    #[test]
    fn decode_to_slice_checks_dst_len() {
        let mut short = [0u8; 1];
        assert_eq!(
            decode_upper_to_slice(b"ABCD", &mut short),
            Err(Error::LengthInvalid)
        );

        let mut long = [0u8; 4];
        assert_eq!(decode_upper_to_slice(b"ABCD", &mut long), Ok(2));
        assert_eq!(long, [0xAB, 0xCD, 0, 0]);
    }
}
