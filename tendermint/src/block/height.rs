use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use tendermint_proto::Protobuf;

use crate::{error::Error, prelude::*};

/// Block height for a particular chain (i.e. number of blocks created since
/// the chain began)
///
/// A height of 0 represents a chain which has not yet produced a block.
#[derive(Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Height(u64);

impl Protobuf<i64> for Height {}

impl TryFrom<i64> for Height {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Height(value.try_into().map_err(Error::negative_height)?))
    }
}

impl From<Height> for i64 {
    fn from(value: Height) -> Self {
        value.value() as i64 // does not overflow. The value is <= i64::MAX
    }
}

impl TryFrom<u64> for Height {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        // Make sure the u64 value can be converted safely to i64
        let _ival: i64 = value.try_into().map_err(Error::integer_overflow)?;

        Ok(Height(value))
    }
}

impl From<Height> for u64 {
    fn from(value: Height) -> Self {
        value.value()
    }
}

impl From<u32> for Height {
    fn from(value: u32) -> Self {
        Height(value as u64)
    }
}

impl From<u16> for Height {
    fn from(value: u16) -> Self {
        Height(value as u64)
    }
}

impl From<u8> for Height {
    fn from(value: u8) -> Self {
        Height(value as u64)
    }
}

// Allow comparing a `Height` directly to unsigned integer types without going
// through `.value()`. `Height` is a newtype around `u64`, so equality reduces
// to the inner value comparison. Both directions are provided so `height == 5`
// and `5 == height` work, which matters in generic code and assertions.

impl PartialEq<u64> for Height {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Height> for u64 {
    fn eq(&self, other: &Height) -> bool {
        *self == other.0
    }
}

impl PartialEq<u32> for Height {
    fn eq(&self, other: &u32) -> bool {
        self.0 == u64::from(*other)
    }
}

impl PartialEq<Height> for u32 {
    fn eq(&self, other: &Height) -> bool {
        u64::from(*self) == other.0
    }
}

impl PartialEq<u16> for Height {
    fn eq(&self, other: &u16) -> bool {
        self.0 == u64::from(*other)
    }
}

impl PartialEq<Height> for u16 {
    fn eq(&self, other: &Height) -> bool {
        u64::from(*self) == other.0
    }
}

impl PartialEq<u8> for Height {
    fn eq(&self, other: &u8) -> bool {
        self.0 == u64::from(*other)
    }
}

impl PartialEq<Height> for u8 {
    fn eq(&self, other: &Height) -> bool {
        u64::from(*self) == other.0
    }
}

impl Height {
    /// Get inner integer value. Alternative to `.0` or `.into()`
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Increment the block height by 1
    pub fn increment(self) -> Self {
        Height::try_from(self.0.checked_add(1).expect("height overflow")).unwrap()
    }
}

impl Debug for Height {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "block::Height({})", self.0)
    }
}

impl Default for Height {
    fn default() -> Self {
        Height(1)
    }
}

impl Display for Height {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Height {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Height::try_from(
            s.parse::<u64>()
                .map_err(|e| Error::parse_int(s.to_string(), e))?,
        )
    }
}

impl<'de> Deserialize<'de> for Height {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{e}")))
    }
}

impl Serialize for Height {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        i64::from(*self).to_string().serialize(serializer)
    }
}

/// Parse `block::Height` from a type
pub trait ParseHeight {
    /// Parse `block::Height`, or return an `Error` if parsing failed
    fn parse_block_height(&self) -> Result<Height, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_by_one() {
        assert_eq!(Height::default().increment().value(), 2);
    }

    #[test]
    fn avoid_try_unwrap_dance() {
        assert_eq!(
            Height::try_from(2_u64).unwrap().value(),
            Height::from(2_u32).value()
        );
    }

    #[test]
    fn partial_eq_with_u64() {
        let h = Height::try_from(42_u64).unwrap();
        assert!(h == 42_u64);
        assert!(42_u64 == h);
        assert!(h != 43_u64);
        assert!(43_u64 != h);
    }

    #[test]
    fn partial_eq_with_smaller_uints() {
        let h = Height::try_from(7_u64).unwrap();
        assert!(h == 7_u32);
        assert!(7_u32 == h);
        assert!(h == 7_u16);
        assert!(7_u16 == h);
        assert!(h == 7_u8);
        assert!(7_u8 == h);
    }

    #[test]
    fn partial_eq_no_truncation() {
        // A Height larger than u32::MAX must compare unequal to any u32 value,
        // including u32::MAX itself, rather than wrapping/truncating.
        let h = Height::try_from(u64::from(u32::MAX) + 1).unwrap();
        assert!(h != u32::MAX);
        assert!(u32::MAX != h);
    }
}
