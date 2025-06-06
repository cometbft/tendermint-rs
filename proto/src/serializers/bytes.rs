//! Serialize/deserialize bytes (`Vec<u8>`) type

/// Serialize into hexstring, deserialize from hexstring
pub mod hexstring {
    use serde::{Deserialize, Deserializer, Serializer};
    use subtle_encoding::hex;

    use crate::prelude::*;
    use crate::serializers::cow_str::CowStr;

    /// Deserialize a hex-encoded string into `Vec<u8>`
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: From<Vec<u8>>,
    {
        let string = Option::<CowStr<'_>>::deserialize(deserializer)?.unwrap_or_default();
        hex::decode_upper(&string)
            .or_else(|_| hex::decode(&string))
            .map(Into::into)
            .map_err(serde::de::Error::custom)
    }

    /// Serialize from a byte slice into a hex-encoded string.
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        let hex_bytes = hex::encode_upper(value.as_ref());
        let hex_string = String::from_utf8(hex_bytes).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&hex_string)
    }
}

/// Serialize into base64string, deserialize from base64string
pub mod base64string {
    use serde::{Deserialize, Deserializer, Serializer};
    use subtle_encoding::base64;

    use crate::prelude::*;
    use crate::serializers::cow_str::CowStr;

    /// Deserialize base64string into `Vec<u8>`
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        Vec<u8>: Into<T>,
    {
        let s = Option::<CowStr<'_>>::deserialize(deserializer)?.unwrap_or_default();
        let v = base64::decode(s).map_err(serde::de::Error::custom)?;
        Ok(v.into())
    }

    /// Deserialize base64string into String
    pub fn deserialize_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<CowStr<'_>>::deserialize(deserializer)?.unwrap_or_default();
        String::from_utf8(base64::decode(s).map_err(serde::de::Error::custom)?)
            .map_err(serde::de::Error::custom)
    }

    /// Serialize from T into base64string
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        let base64_bytes = base64::encode(value.as_ref());
        let base64_string = String::from_utf8(base64_bytes).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&base64_string)
    }
}

/// Serialize into and deserialize from a sequence of _base64string_.
pub mod vec_base64string {
    use serde::{Deserialize, Deserializer, Serializer};
    use subtle_encoding::base64;

    use crate::prelude::*;
    use crate::serializers::cow_str::CowStr;

    /// Deserialize array into `Vec<Vec<u8>>`
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<Vec<CowStr<'_>>>::deserialize(deserializer)?
            .unwrap_or_default()
            .into_iter()
            .map(|s| base64::decode(s).map_err(serde::de::Error::custom))
            .collect()
    }

    /// Serialize from `Vec<T>` into `Vec<base64string>`
    pub fn serialize<S, T>(value: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        let base64_strings = value
            .iter()
            .map(|v| {
                String::from_utf8(base64::encode(v.as_ref())).map_err(serde::ser::Error::custom)
            })
            .collect::<Result<Vec<String>, S::Error>>()?;
        serializer.collect_seq(base64_strings)
    }
}

/// Serialize into and deserialize from an optional _base64string_.
pub mod option_base64string {
    use serde::{Deserialize, Deserializer, Serializer};
    use subtle_encoding::base64;

    use crate::prelude::*;
    use crate::serializers::cow_str::CowStr;

    /// Deserialize `Option<base64string>` into `Vec<u8>` or null
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<CowStr<'_>>::deserialize(deserializer)?.unwrap_or_default();
        base64::decode(s).map_err(serde::de::Error::custom)
    }

    /// Serialize from `T` into `Option<base64string>`
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        let base64_bytes = base64::encode(value.as_ref());
        let base64_string = String::from_utf8(base64_bytes).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&base64_string)
    }
}

/// Serialize into string, deserialize from string
pub mod string {
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::prelude::*;
    use crate::serializers::cow_str::CowStr;

    /// Deserialize string into `Vec<u8>`
    #[allow(dead_code)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = Option::<CowStr<'_>>::deserialize(deserializer)?.unwrap_or_default();
        Ok(string.as_bytes().to_vec())
    }

    /// Serialize from `T` into string
    #[allow(dead_code)]
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        let string =
            String::from_utf8(value.as_ref().to_vec()).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&string)
    }
}
