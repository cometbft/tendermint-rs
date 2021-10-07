//! Timestamps used by Tendermint blockchains

use chrono::{DateTime, LocalResult, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use core::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time::Duration;
use tendermint_proto::google::protobuf::Timestamp;
use tendermint_proto::serializers::timestamp;
use tendermint_proto::Protobuf;

use crate::error::Error;

/// Tendermint timestamps
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#time>
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "Timestamp")]
pub struct Time(pub DateTime<Utc>);

impl Protobuf<Timestamp> for Time {}

impl TryFrom<Timestamp> for Time {
    type Error = Error;

    fn try_from(value: Timestamp) -> Result<Self, Error> {
        let nanos = value.nanos.try_into().map_err(Error::timestamp_overflow)?;
        Time::from_unix_timestamp(value.seconds, nanos)
    }
}

impl From<Time> for Timestamp {
    fn from(value: Time) -> Self {
        // Subsecond nanoseconds returned by timestamp_subsec_nanos should have a value
        // between 0 and 999,999,999. So that shouldn't cause an overflow when converting
        // from u32 to i32. However in case there is an unexpected conversion error,
        // we default to 0, and hopefully does not cause any undefined behavior.
        let nanos = value.0.timestamp_subsec_nanos().try_into().unwrap_or(0);

        Timestamp {
            seconds: value.0.timestamp(),
            nanos,
        }
    }
}

impl Time {
    /// Get the unix epoch ("1970-01-01 00:00:00 UTC") as a [`Time`]
    pub fn unix_epoch() -> Self {
        Time(Utc.timestamp(0, 0))
    }

    pub fn from_unix_timestamp(secs: i64, nanos: u32) -> Result<Self, Error> {
        match Utc.timestamp_opt(secs, nanos) {
            LocalResult::Single(time) => Ok(Time(time)),
            _ => Err(Error::timestamp_conversion()),
        }
    }

    /// Calculate the amount of time which has passed since another [`Time`]
    /// as a [`std::time::Duration`]
    pub fn duration_since(&self, other: Time) -> Result<Duration, Error> {
        self.0
            .signed_duration_since(other.0)
            .to_std()
            .map_err(|_| Error::duration_out_of_range())
    }

    /// Parse [`Time`] from an RFC 3339 date
    pub fn parse_from_rfc3339(s: &str) -> Result<Time, Error> {
        let date = DateTime::parse_from_rfc3339(s)
            .map_err(Error::chrono_parse)?
            .with_timezone(&Utc);
        Ok(Time(date))
    }

    /// Return an RFC 3339 and ISO 8601 date and time string with 6 subseconds digits and Z.
    pub fn as_rfc3339(&self) -> String {
        timestamp::as_rfc3339_nanos(&self.0)
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.as_rfc3339())
    }
}

impl FromStr for Time {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Time::parse_from_rfc3339(s)
    }
}

impl From<DateTime<Utc>> for Time {
    fn from(t: DateTime<Utc>) -> Time {
        Time(t)
    }
}

impl From<Time> for DateTime<Utc> {
    fn from(t: Time) -> DateTime<Utc> {
        t.0
    }
}

impl Add<Duration> for Time {
    type Output = Result<Self, Error>;

    fn add(self, rhs: Duration) -> Self::Output {
        let duration =
            chrono::Duration::from_std(rhs).map_err(|_| Error::duration_out_of_range())?;

        let res = self
            .0
            .checked_add_signed(duration)
            .ok_or_else(Error::duration_out_of_range)?;

        Ok(Time(res))
    }
}

impl Sub<Duration> for Time {
    type Output = Result<Self, Error>;

    fn sub(self, rhs: Duration) -> Self::Output {
        let duration =
            chrono::Duration::from_std(rhs).map_err(|_| Error::duration_out_of_range())?;

        let res = self
            .0
            .checked_sub_signed(duration)
            .ok_or_else(Error::duration_out_of_range)?;

        Ok(Time(res))
    }
}

/// Parse [`Time`] from a type
pub trait ParseTimestamp {
    /// Parse [`Time`], or return an [`Error`] if parsing failed
    fn parse_timestamp(&self) -> Result<Time, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{prelude::*, sample::select};
    use tendermint_pbt_gen as pbt;

    // We want to make sure that these timestamps specifically get tested.
    fn particular_rfc3339_timestamps() -> impl Strategy<Value = String> {
        let strs: Vec<String> = vec![
            "2020-09-14T16:33:54.21191421Z",
            "2020-09-14T16:33:00Z",
            "2020-09-14T16:33:00.1Z",
            "2020-09-14T16:33:00.211914212Z",
            "1970-01-01T00:00:00Z",
            "2021-01-07T20:25:56.0455760Z",
            "2021-01-07T20:25:57.039219Z",
            "2021-01-07T20:25:58.03562100Z",
            "2021-01-07T20:25:59.000955200Z",
            "2021-01-07T20:26:04.0121030Z",
            "2021-01-07T20:26:05.005096Z",
            "2021-01-07T20:26:09.08488400Z",
            "2021-01-07T20:26:11.0875340Z",
            "2021-01-07T20:26:12.078268Z",
            "2021-01-07T20:26:13.08074100Z",
            "2021-01-07T20:26:15.079663000Z",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        select(strs)
    }

    proptest! {
        #[test]
        fn can_parse_rfc3339_timestamps(stamp in pbt::time::arb_rfc3339_timestamp()) {
            prop_assert!(stamp.parse::<Time>().is_ok())
        }

        #[test]
        fn serde_from_value_is_the_inverse_of_to_value_within_reasonable_time_range(
            datetime in pbt::time::arb_datetime()
        ) {
            // If `from_value` is the inverse of `to_value`, then it will always
            // map the JSON `encoded_time` to back to the inital `time`.
            let time: Time = datetime.into();
            let json_encoded_time = serde_json::to_value(&time).unwrap();
            let decoded_time: Time = serde_json::from_value(json_encoded_time).unwrap();
            prop_assert_eq!(time, decoded_time);
        }

        #[test]
        fn serde_of_rfc3339_timestamps_is_safe(
            stamp in prop_oneof![
                pbt::time::arb_rfc3339_timestamp(),
                particular_rfc3339_timestamps(),
            ]
        ) {
            // ser/de of rfc3339 timestamps is safe if it never panics.
            // This differes from the the inverse test in that we are testing on
            // arbitrarily generated textual timestamps, rather than times in a
            // range. Tho we do incidentally test the inversion as well.
            let time: Time = stamp.parse().unwrap();
            let json_encoded_time = serde_json::to_value(&time).unwrap();
            let decoded_time: Time = serde_json::from_value(json_encoded_time).unwrap();
            prop_assert_eq!(time, decoded_time);
        }
    }
}
