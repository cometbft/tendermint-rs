//! Serialize/deserialize Timestamp type from and into string:

use crate::google::protobuf::Timestamp;
use crate::prelude::*;
use serde::de::Error as _;
use serde::ser::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::format_description::well_known::Rfc3339 as Rfc3339Format;
use time::macros::{format_description, offset};
use time::OffsetDateTime;

/// Helper struct to serialize and deserialize Timestamp into an RFC3339-compatible string
/// This is required because the serde `with` attribute is only available to fields of a struct but
/// not the whole struct.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rfc3339(#[serde(with = "crate::serializers::timestamp")] Timestamp);

impl From<Timestamp> for Rfc3339 {
    fn from(value: Timestamp) -> Self {
        Rfc3339(value)
    }
}
impl From<Rfc3339> for Timestamp {
    fn from(value: Rfc3339) -> Self {
        value.0
    }
}

/// Deserialize string into Timestamp
pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    let value_string = String::deserialize(deserializer)?;
    let value_datetime =
        OffsetDateTime::parse(&value_string, &Rfc3339Format).map_err(D::Error::custom)?;
    let total_nanos = value_datetime.unix_timestamp_nanos();
    Ok(Timestamp {
        seconds: total_nanos.div_euclid(1_000_000_000) as _,
        nanos: total_nanos.rem_euclid(1_000_000_000) as _,
    })
}

/// Serialize from Timestamp into string
pub fn serialize<S>(value: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.nanos < 0 {
        return Err(S::Error::custom("invalid nanoseconds in time"));
    }
    let total_nanos = value.seconds as i128 * 1_000_000_000 + value.nanos as i128;
    let datetime = OffsetDateTime::from_unix_timestamp_nanos(total_nanos)
        .map_err(|_| S::Error::custom("invalid time"))?;
    to_rfc3339_nanos(datetime).serialize(serializer)
}

/// Serialization helper for converting an [`OffsetDateTime`] object to a string.
///
/// This reproduces the behavior of Go's `time.RFC3339Nano` format,
/// ie. a RFC3339 date-time with left-padded subsecond digits without
///     trailing zeros and no trailing dot.
pub fn to_rfc3339_nanos(t: OffsetDateTime) -> String {
    let t = t.to_offset(offset!(UTC));
    let format = if t.nanosecond() == 0 {
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z")
    } else {
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]Z")
    };

    t.format(format).unwrap()
}

#[allow(warnings)]
#[cfg(test)]
mod test {
    use super::*;
    use crate::google::protobuf::Timestamp;
    use serde::{Deserialize, Serialize};

    // The Go code with which the following timestamps
    // were tested is as follows:
    //
    // ```go
    // package main
    //
    // import (
    //     "fmt"
    //     "time"
    // )
    //
    // func main() {
    //     timestamps := []string{
    //         "1970-01-01T00:00:00Z",
    //         "0001-01-01T00:00:00Z",
    //         "2020-09-14T16:33:00Z",
    //         "2020-09-14T16:33:00.1Z",
    //         "2020-09-14T16:33:00.211914212Z",
    //         "2020-09-14T16:33:54.21191421Z",
    //         "2021-01-07T20:25:56.045576Z",
    //         "2021-01-07T20:25:57.039219Z",
    //         "2021-01-07T20:26:05.00509Z",
    //         "2021-01-07T20:26:05.005096Z",
    //         "2021-01-07T20:26:05.0005096Z",
    //     }
    //     for _, timestamp := range timestamps {
    //         ts, err := time.Parse(time.RFC3339Nano, timestamp)
    //         if err != nil {
    //             panic(err)
    //         }
    //         tss := ts.Format(time.RFC3339Nano)
    //         if timestamp != tss {
    //             panic(fmt.Sprintf("\nExpected : %s\nActual   : %s", timestamp, tss))
    //         }
    //     }
    //     fmt.Println("All good!")
    // }
    // ```
    #[test]
    fn json_timestamp_precision() {
        let test_timestamps = vec![
            "1970-01-01T00:00:00Z",
            "0001-01-01T00:00:00Z",
            "2020-09-14T16:33:00Z",
            "2020-09-14T16:33:00.1Z",
            "2020-09-14T16:33:00.211914212Z",
            "2020-09-14T16:33:54.21191421Z",
            "2021-01-07T20:25:56.045576Z",
            "2021-01-07T20:25:57.039219Z",
            "2021-01-07T20:26:05.00509Z",
            "2021-01-07T20:26:05.005096Z",
            "2021-01-07T20:26:05.0005096Z",
        ];

        for timestamp in test_timestamps {
            let json = format!("\"{}\"", timestamp);
            let rfc = serde_json::from_str::<Rfc3339>(&json).unwrap();
            assert_eq!(json, serde_json::to_string(&rfc).unwrap());
        }
    }
}
