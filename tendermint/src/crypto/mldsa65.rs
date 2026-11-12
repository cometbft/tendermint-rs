//! ML-DSA-65 (FIPS 204) validator keys, as used by CometBFT v0.40
//! (and accepted for verification by v0.38.26 and later).
//!
//! Keys and signatures use the FIPS 204 packed encodings. Signatures are
//! "pure" ML-DSA over an empty context string.

use core::cmp::Ordering;

use crate::{prelude::*, Error};

/// Length of an encoded ML-DSA-65 public key, in bytes.
pub const PUBLIC_KEY_LENGTH: usize = 1952;

/// Length of an encoded ML-DSA-65 signature, in bytes.
pub const SIGNATURE_LENGTH: usize = crate::signature::MLDSA65_SIGNATURE_LENGTH;

/// An ML-DSA-65 public key in its FIPS 204 packed encoding (`pkEncode`).
///
/// Any byte string of the right length is a valid encoding, so construction
/// only checks the length; the key is expanded when a signature is verified.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct VerificationKey([u8; PUBLIC_KEY_LENGTH]);

impl VerificationKey {
    /// The encoded key bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&'_ [u8]> for VerificationKey {
    type Error = Error;

    fn try_from(slice: &'_ [u8]) -> Result<Self, Self::Error> {
        let bytes: [u8; PUBLIC_KEY_LENGTH] = slice.try_into().map_err(|_| {
            Error::invalid_key(format!(
                "invalid ML-DSA-65 key length: expected {PUBLIC_KEY_LENGTH}, got {}",
                slice.len()
            ))
        })?;
        Ok(Self(bytes))
    }
}

impl Ord for VerificationKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0[..].cmp(&other.0[..])
    }
}

impl PartialOrd for VerificationKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl core::fmt::Display for VerificationKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl core::fmt::Debug for VerificationKey {
    // Abbreviated: the full key is 3904 hex characters.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MlDsa65(")?;
        for byte in &self.0[..8] {
            write!(f, "{byte:02x}")?;
        }
        write!(f, "..)")
    }
}

/// Verify a pure ML-DSA-65 signature (FIPS 204 Algorithm 3, empty context).
pub(crate) fn verify(
    key: &VerificationKey,
    msg: &[u8],
    signature: &[u8],
) -> Result<(), crate::crypto::signature::Error> {
    use crate::crypto::signature::Error as VerifyError;
    use ml_dsa::{EncodedVerifyingKey, MlDsa65, Signature, VerifyingKey};

    let encoded = EncodedVerifyingKey::<MlDsa65>::try_from(&key.0[..])
        .map_err(|_| VerifyError::MalformedPublicKey)?;
    let vk = VerifyingKey::<MlDsa65>::decode(&encoded);
    let sig =
        Signature::<MlDsa65>::try_from(signature).map_err(|_| VerifyError::MalformedSignature)?;
    if vk.verify_with_context(msg, &[], &sig) {
        Ok(())
    } else {
        Err(VerifyError::VerificationFailed)
    }
}
