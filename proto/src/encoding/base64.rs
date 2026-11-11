//! Standard (RFC 4648, non-URL-safe, padded) Base64 in (almost)
//! data-independent constant time.
//!
//! Character set: `[A-Z]`, `[a-z]`, `[0-9]`, `+`, `/`, padded with `=`.

use zeroize::Zeroize;

use super::{ends_with_whitespace, Error};
use crate::prelude::*;

/// Encode `bytes` as padded Base64.
pub fn encode<B: AsRef<[u8]>>(bytes: B) -> Vec<u8> {
    let src = bytes.as_ref();
    let mut dst = vec![0u8; encoded_len(src.len())];

    let mut src_chunks = src.chunks_exact(3);
    let mut dst_chunks = dst.chunks_exact_mut(4);
    for (src_chunk, dst_chunk) in src_chunks.by_ref().zip(dst_chunks.by_ref()) {
        encode_3bytes(src_chunk, dst_chunk);
    }

    let rem = src_chunks.remainder();
    if let Some(dst_chunk) = dst_chunks.next() {
        let mut tmp = [0u8; 3];
        tmp[..rem.len()].copy_from_slice(rem);
        encode_3bytes(&tmp, dst_chunk);
        tmp.zeroize();

        dst_chunk[3] = b'=';
        if rem.len() == 1 {
            dst_chunk[2] = b'=';
        }
    }

    dst
}

/// Decode padded Base64.
///
/// Missing padding is tolerated, e.g. `"AB"` decodes the same as `"AB=="`.
pub fn decode<B: AsRef<[u8]>>(encoded: B) -> Result<Vec<u8>, Error> {
    let src = encoded.as_ref();
    let mut dst = vec![0u8; decoded_len(src)];
    let len = decode_to_slice(src, &mut dst)?;
    debug_assert_eq!(len, dst.len());
    Ok(dst)
}

/// Decode padded Base64 from `src` into the start of `dst`, returning the
/// number of bytes written.
///
/// Fails with [`Error::LengthInvalid`] if `dst` is too short.
pub fn decode_to_slice(src: &[u8], dst: &mut [u8]) -> Result<usize, Error> {
    if ends_with_whitespace(src) {
        return Err(Error::TrailingWhitespace);
    }

    if decoded_len(src) > dst.len() {
        return Err(Error::LengthInvalid);
    }

    // Every quantum except the last must be 4 non-padding characters. The last
    // quantum (1 to 4 characters) is handled separately below.
    let last_quantum_len = match src.len() % 4 {
        0 => src.len().min(4),
        n => n,
    };
    let (full, last) = src.split_at(src.len() - last_quantum_len);

    let mut err: isize = 0;
    let mut dst_offset: usize = 0;

    for src_chunk in full.chunks_exact(4) {
        // `decoded_len` counts every character in `full`, so this can only
        // fail when `last` is made of padding only, which is invalid anyway.
        let dst_chunk = dst
            .get_mut(dst_offset..dst_offset + 3)
            .ok_or(Error::EncodingInvalid)?;
        err |= decode_3bytes(src_chunk, dst_chunk);
        dst_offset += 3;
    }

    if !last.is_empty() {
        // Number of non-padding characters in the last quantum. Everything
        // after the first `=` must be padding as well.
        let data_len = last.iter().position(|&c| c == b'=').unwrap_or(last.len());
        if data_len < 2 || last[data_len..].iter().any(|&c| c != b'=') {
            return Err(Error::EncodingInvalid);
        }

        let mut tmp_in = [b'A'; 4];
        let mut tmp_out = [0u8; 3];
        tmp_in[..data_len].copy_from_slice(&last[..data_len]);
        err |= decode_3bytes(&tmp_in, &mut tmp_out);
        tmp_in.zeroize();

        let out_len = data_len - 1;
        let result = dst
            .get_mut(dst_offset..dst_offset + out_len)
            .ok_or(Error::EncodingInvalid)
            .map(|dst_chunk| dst_chunk.copy_from_slice(&tmp_out[..out_len]));
        tmp_out.zeroize();
        result?;

        dst_offset += out_len;
    }

    if err == 0 {
        Ok(dst_offset)
    } else {
        Err(Error::EncodingInvalid)
    }
}

/// Length of the padded Base64 encoding of `len` bytes.
fn encoded_len(len: usize) -> usize {
    ((len * 4 / 3) + 3) & !3
}

/// Number of bytes that `src` decodes to, ignoring trailing padding.
fn decoded_len(src: &[u8]) -> usize {
    // Mirrors subtle-encoding: the first character is never counted as padding.
    let pad_count = src.iter().skip(1).rev().take_while(|&&c| c == b'=').count();
    (src.len() - pad_count) * 3 / 4
}

// Base64 character set:
// [A-Z]      [a-z]      [0-9]      +     /
// 0x41-0x5a, 0x61-0x7a, 0x30-0x39, 0x2b, 0x2f

#[inline]
fn encode_3bytes(src: &[u8], dst: &mut [u8]) {
    let b0 = src[0] as isize;
    let b1 = src[1] as isize;
    let b2 = src[2] as isize;

    dst[0] = encode_6bits(b0 >> 2);
    dst[1] = encode_6bits(((b0 << 4) | (b1 >> 4)) & 63);
    dst[2] = encode_6bits(((b1 << 2) | (b2 >> 6)) & 63);
    dst[3] = encode_6bits(b2 & 63);
}

#[inline]
fn encode_6bits(src: isize) -> u8 {
    let mut diff = 0x41isize;

    // if (in > 25) diff += 0x61 - 0x41 - 26; // 6
    diff += ((25isize - src) >> 8) & 6;

    // if (in > 51) diff += 0x30 - 0x61 - 26; // -75
    diff -= ((51isize - src) >> 8) & 75;

    // if (in > 61) diff += 0x2b - 0x30 - 10; // -15
    diff -= ((61isize - src) >> 8) & 15;

    // if (in > 62) diff += 0x2f - 0x2b - 1; // 3
    diff += ((62isize - src) >> 8) & 3;

    (src + diff) as u8
}

/// Decode 4 characters into 3 bytes. Returns non-zero if any character is
/// outside the Base64 alphabet.
#[inline]
fn decode_3bytes(src: &[u8], dst: &mut [u8]) -> isize {
    let c0 = decode_6bits(src[0]);
    let c1 = decode_6bits(src[1]);
    let c2 = decode_6bits(src[2]);
    let c3 = decode_6bits(src[3]);

    dst[0] = ((c0 << 2) | (c1 >> 4)) as u8;
    dst[1] = ((c1 << 4) | (c2 >> 2)) as u8;
    dst[2] = ((c2 << 6) | c3) as u8;

    ((c0 | c1 | c2 | c3) >> 8) & 1
}

#[inline]
fn decode_6bits(src: u8) -> isize {
    let ch = src as isize;
    let mut ret: isize = -1;

    // if (ch > 0x40 && ch < 0x5b) ret += ch - 0x41 + 1; // -64
    ret += (((64isize - ch) & (ch - 91isize)) >> 8) & (ch - 64isize);

    // if (ch > 0x60 && ch < 0x7b) ret += ch - 0x61 + 26 + 1; // -70
    ret += (((96isize - ch) & (ch - 123isize)) >> 8) & (ch - 70isize);

    // if (ch > 0x2f && ch < 0x3a) ret += ch - 0x30 + 52 + 1; // 5
    ret += (((47isize - ch) & (ch - 58isize)) >> 8) & (ch + 5isize);

    // if (ch == 0x2b) ret += 62 + 1;
    ret += (((42isize - ch) & (ch - 44isize)) >> 8) & 63;

    // if (ch == 0x2f) ret += 63 + 1;
    ret + ((((46isize - ch) & (ch - 48isize)) >> 8) & 64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (raw, base64) test vectors
    const VECTORS: &[(&[u8], &[u8])] = &[
        (b"", b""),
        (b"\0", b"AA=="),
        (b"***", b"Kioq"),
        (b"\x01\x02\x03\x04", b"AQIDBA=="),
        (b"\xAD\xAD\xAD\xAD\xAD", b"ra2tra0="),
        (b"\xFF\xFF\xFF\xFF\xFF", b"//////8="),
        (
            b"\x40\xC1\x3F\xBD\x05\x4C\x72\x2A\xA3\xC2\xF2\x11\x73\xC0\x69\xEA\
              \x49\x7D\x35\x29\x6B\xCC\x24\x65\xF6\xF9\xD0\x41\x08\x7B\xD7\xA9",
            b"QME/vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k=",
        ),
    ];

    #[test]
    fn encode_test_vectors() {
        for (raw, b64) in VECTORS {
            assert_eq!(encode(raw), *b64);
        }
    }

    #[test]
    fn decode_test_vectors() {
        for (raw, b64) in VECTORS {
            assert_eq!(decode(b64).unwrap(), *raw);
        }
    }

    #[test]
    fn encode_and_decode_various_lengths() {
        let data: Vec<u8> = (0..=255).collect();
        for i in 0..data.len() {
            let encoded = encode(&data[..i]);
            assert_eq!(encoded.len(), encoded_len(i));
            assert_eq!(decode(encoded).unwrap(), &data[..i]);
        }
    }

    #[test]
    fn missing_padding_is_tolerated() {
        assert_eq!(decode("AB").unwrap(), [0]);
        assert_eq!(decode("ABC").unwrap(), [0, 16]);
        assert_eq!(decode("AB=").unwrap(), [0]);
    }

    #[test]
    fn trailing_whitespace() {
        assert_eq!(
            decode(&b"QME/vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k=\n"[..]),
            Err(Error::TrailingWhitespace)
        );
    }

    /// subtle-encoding 0.5.1 panicked on these (or, for `AB=C`, returned
    /// garbage in release builds).
    #[test]
    fn malformed_padding_is_an_error() {
        for input in [
            "=",
            "==",
            "====",
            "A",
            "A===",
            "AAAA=",
            "AAAA====",
            "AAAA=====",
            "AB=C",
            "=AAA",
            "AA=A",
            "AAA=AAAA",
        ] {
            assert_eq!(decode(input), Err(Error::EncodingInvalid), "{input:?}");
        }
    }

    #[test]
    fn invalid_characters() {
        assert_eq!(decode("AA-A"), Err(Error::EncodingInvalid));
        assert_eq!(decode("AA A"), Err(Error::EncodingInvalid));
    }

    #[test]
    fn decode_to_slice_checks_dst_len() {
        let mut short = [0u8; 2];
        assert_eq!(
            decode_to_slice(b"Kioq", &mut short),
            Err(Error::LengthInvalid)
        );

        let mut long = [0u8; 5];
        assert_eq!(decode_to_slice(b"Kioq", &mut long), Ok(3));
        assert_eq!(long, [b'*', b'*', b'*', 0, 0]);
    }
}
