//! Hex, Base64 and Bech32 encoding.
//!
//! The hex and Base64 implementations run in (almost) data-independent constant
//! time, which makes them suitable for secret material such as private keys.
//!
//! This module is a trimmed-down copy of the parts of
//! [`subtle-encoding`](https://github.com/iqlusioninc/crates/tree/main/subtle-encoding) v0.5.1
//! that tendermint-rs uses. `subtle-encoding` is Copyright (c) iqlusion and is dual licensed
//! under Apache 2.0 and MIT. The constant-time hex and Base64 routines were adapted from
//! <https://github.com/Sc00bz/ConstTimeEncoding>, Copyright (c) 2014 Steve "Sc00bz" Thomas,
//! with permission.
//!
//! CAUTION: There are no guarantees for backwards compatibility, this module should be considered
//! an internal implementation detail which can vanish without further warning. Use at your own
//! risk.

use core::fmt;

pub mod base64;
pub mod bech32;
pub mod hex;

/// Encoding or decoding error
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// The input contains characters outside the expected alphabet or is malformed
    EncodingInvalid,

    /// The input or output buffer has an invalid length
    LengthInvalid,

    /// The input ends with whitespace
    TrailingWhitespace,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::EncodingInvalid => "bad encoding",
            Error::LengthInvalid => "invalid length",
            Error::TrailingWhitespace => "trailing whitespace",
        })
    }
}

impl core::error::Error for Error {}

/// Whether the last byte of `src`, read as a Latin-1 character, is whitespace.
fn ends_with_whitespace(src: &[u8]) -> bool {
    src.last()
        .is_some_and(|byte| char::from(*byte).is_whitespace())
}
