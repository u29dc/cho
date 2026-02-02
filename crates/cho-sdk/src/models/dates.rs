//! Xero Microsoft-style date newtypes with custom serde.
//!
//! Xero uses three date formats in API responses:
//! - `x-is-msdate: true` → `/Date(epoch_ms+offset)/` → [`MsDate`] wrapping [`NaiveDate`]
//! - `x-is-msdate-time: true` → `/Date(epoch_ms)/` → [`MsDateTime`] wrapping [`DateTime<Utc>`]
//! - `format: date` → ISO `YYYY-MM-DD` → plain [`NaiveDate`] (handled by chrono's default serde)

use std::fmt;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Regex pattern matching Xero MS Date format: `/Date(epoch_ms)/ or /Date(epoch_ms+offset)/`
const MS_DATE_PATTERN: &str = r"/Date\((-?\d+)(\+\d{4})?\)/";

/// A date newtype wrapping [`NaiveDate`] that deserializes from Xero's
/// `/Date(epoch_ms+offset)/` format and serializes to `YYYY-MM-DD`.
///
/// Used for fields marked `x-is-msdate: true` in the Xero OpenAPI spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MsDate(pub NaiveDate);

/// A datetime newtype wrapping [`DateTime<Utc>`] that deserializes from Xero's
/// `/Date(epoch_ms)/` format.
///
/// Used for fields marked `x-is-msdate-time: true` in the Xero OpenAPI spec.
/// These are typically read-only audit timestamps (e.g., `UpdatedDateUTC`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MsDateTime(pub DateTime<Utc>);

impl MsDate {
    /// Create a new [`MsDate`] from a [`NaiveDate`].
    pub fn new(date: NaiveDate) -> Self {
        Self(date)
    }

    /// Returns the inner [`NaiveDate`].
    pub fn into_inner(self) -> NaiveDate {
        self.0
    }
}

impl MsDateTime {
    /// Create a new [`MsDateTime`] from a [`DateTime<Utc>`].
    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Returns the inner [`DateTime<Utc>`].
    pub fn into_inner(self) -> DateTime<Utc> {
        self.0
    }
}

impl fmt::Display for MsDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

impl fmt::Display for MsDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%dT%H:%M:%S%.3fZ"))
    }
}

impl From<NaiveDate> for MsDate {
    fn from(date: NaiveDate) -> Self {
        Self(date)
    }
}

impl From<MsDate> for NaiveDate {
    fn from(ms: MsDate) -> Self {
        ms.0
    }
}

impl From<DateTime<Utc>> for MsDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<MsDateTime> for DateTime<Utc> {
    fn from(ms: MsDateTime) -> Self {
        ms.0
    }
}

/// Parse epoch milliseconds from a Xero MS Date string.
///
/// Accepts formats: `/Date(1539993600000+0000)/` and `/Date(1573755038314)/`
fn parse_ms_date_epoch(s: &str) -> Option<i64> {
    let re = Regex::new(MS_DATE_PATTERN).ok()?;
    let caps = re.captures(s)?;
    caps.get(1)?.as_str().parse::<i64>().ok()
}

impl Serialize for MsDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as YYYY-MM-DD for request bodies
        serializer.serialize_str(&self.0.format("%Y-%m-%d").to_string())
    }
}

impl<'de> Deserialize<'de> for MsDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Try MS Date format first: /Date(epoch_ms+offset)/
        if let Some(epoch_ms) = parse_ms_date_epoch(&s) {
            let epoch_secs = epoch_ms / 1000;
            let dt = Utc
                .timestamp_opt(epoch_secs, 0)
                .single()
                .ok_or_else(|| de::Error::custom(format!("invalid epoch timestamp: {epoch_ms}")))?;
            return Ok(MsDate(dt.date_naive()));
        }

        // Fall back to ISO YYYY-MM-DD
        let date = NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(|e| de::Error::custom(format!("invalid date format '{s}': {e}")))?;
        Ok(MsDate(date))
    }
}

impl Serialize for MsDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as ISO 8601
        serializer.serialize_str(&self.0.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
    }
}

impl<'de> Deserialize<'de> for MsDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Try MS Date format: /Date(epoch_ms)/
        if let Some(epoch_ms) = parse_ms_date_epoch(&s) {
            let epoch_secs = epoch_ms / 1000;
            let nanos = ((epoch_ms % 1000) * 1_000_000) as u32;
            let dt = Utc
                .timestamp_opt(epoch_secs, nanos)
                .single()
                .ok_or_else(|| de::Error::custom(format!("invalid epoch timestamp: {epoch_ms}")))?;
            return Ok(MsDateTime(dt));
        }

        // Fall back to ISO 8601
        let dt = s
            .parse::<DateTime<Utc>>()
            .map_err(|e| de::Error::custom(format!("invalid datetime format '{s}': {e}")))?;
        Ok(MsDateTime(dt))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    // ── MsDate deserialize tests ──

    #[test]
    fn ms_date_deserialize_with_offset() {
        let json = r#""/Date(1539993600000+0000)/""#;
        let date: MsDate = serde_json::from_str(json).unwrap();
        assert_eq!(date.0, NaiveDate::from_ymd_opt(2018, 10, 20).unwrap());
    }

    #[test]
    fn ms_date_deserialize_without_offset() {
        let json = r#""/Date(1539993600000)/""#;
        let date: MsDate = serde_json::from_str(json).unwrap();
        assert_eq!(date.0, NaiveDate::from_ymd_opt(2018, 10, 20).unwrap());
    }

    #[test]
    fn ms_date_deserialize_zero_epoch() {
        let json = r#""/Date(0+0000)/""#;
        let date: MsDate = serde_json::from_str(json).unwrap();
        assert_eq!(date.0, NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    }

    #[test]
    fn ms_date_deserialize_negative_epoch() {
        // 1969-12-31 = -86400000 ms
        let json = r#""/Date(-86400000+0000)/""#;
        let date: MsDate = serde_json::from_str(json).unwrap();
        assert_eq!(date.0, NaiveDate::from_ymd_opt(1969, 12, 31).unwrap());
    }

    #[test]
    fn ms_date_deserialize_large_epoch() {
        // 2050-01-01 00:00:00 UTC = 2524608000000 ms
        let json = r#""/Date(2524608000000+0000)/""#;
        let date: MsDate = serde_json::from_str(json).unwrap();
        assert_eq!(date.0, NaiveDate::from_ymd_opt(2050, 1, 1).unwrap());
    }

    #[test]
    fn ms_date_deserialize_iso_fallback() {
        let json = r#""2019-10-31""#;
        let date: MsDate = serde_json::from_str(json).unwrap();
        assert_eq!(date.0, NaiveDate::from_ymd_opt(2019, 10, 31).unwrap());
    }

    #[test]
    fn ms_date_deserialize_with_nonzero_offset() {
        // offset is ignored for date extraction; epoch is always UTC
        let json = r#""/Date(1539993600000+1200)/""#;
        let date: MsDate = serde_json::from_str(json).unwrap();
        assert_eq!(date.0, NaiveDate::from_ymd_opt(2018, 10, 20).unwrap());
    }

    // ── MsDate serialize tests ──

    #[test]
    fn ms_date_serialize_to_iso() {
        let date = MsDate(NaiveDate::from_ymd_opt(2019, 10, 31).unwrap());
        let json = serde_json::to_string(&date).unwrap();
        assert_eq!(json, r#""2019-10-31""#);
    }

    // ── MsDate round-trip tests ──

    #[test]
    fn ms_date_round_trip_ms_format() {
        let json = r#""/Date(1539993600000+0000)/""#;
        let date: MsDate = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&date).unwrap();
        assert_eq!(serialized, r#""2018-10-20""#);
        // Re-deserialize from ISO
        let date2: MsDate = serde_json::from_str(&serialized).unwrap();
        assert_eq!(date, date2);
    }

    #[test]
    fn ms_date_round_trip_negative_epoch() {
        let json = r#""/Date(-86400000+0000)/""#;
        let date: MsDate = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&date).unwrap();
        assert_eq!(serialized, r#""1969-12-31""#);
        let date2: MsDate = serde_json::from_str(&serialized).unwrap();
        assert_eq!(date, date2);
    }

    // ── MsDateTime deserialize tests ──

    #[test]
    fn ms_datetime_deserialize_basic() {
        let json = r#""/Date(1573755038314)/""#;
        let dt: MsDateTime = serde_json::from_str(json).unwrap();
        assert_eq!(dt.0.timestamp(), 1_573_755_038);
        assert_eq!(dt.0.timestamp_subsec_millis(), 314);
    }

    #[test]
    fn ms_datetime_deserialize_zero_epoch() {
        let json = r#""/Date(0)/""#;
        let dt: MsDateTime = serde_json::from_str(json).unwrap();
        assert_eq!(dt.0, Utc.timestamp_opt(0, 0).unwrap());
    }

    #[test]
    fn ms_datetime_deserialize_negative_epoch() {
        let json = r#""/Date(-1000)/""#;
        let dt: MsDateTime = serde_json::from_str(json).unwrap();
        assert_eq!(dt.0.timestamp(), -1);
        assert_eq!(dt.0.timestamp_subsec_millis(), 0);
    }

    #[test]
    fn ms_datetime_deserialize_large_epoch() {
        // 2050-01-01 00:00:00 UTC
        let json = r#""/Date(2524608000000)/""#;
        let dt: MsDateTime = serde_json::from_str(json).unwrap();
        assert_eq!(dt.0.timestamp(), 2_524_608_000);
    }

    #[test]
    fn ms_datetime_deserialize_with_offset() {
        // MsDateTime can also have +0000 offset; epoch is still extracted correctly
        let json = r#""/Date(1573755038314+0000)/""#;
        let dt: MsDateTime = serde_json::from_str(json).unwrap();
        assert_eq!(dt.0.timestamp(), 1_573_755_038);
    }

    // ── MsDateTime serialize tests ──

    #[test]
    fn ms_datetime_serialize_to_iso() {
        let dt = MsDateTime(Utc.timestamp_opt(1_573_755_038, 314_000_000).unwrap());
        let json = serde_json::to_string(&dt).unwrap();
        assert_eq!(json, r#""2019-11-14T18:10:38.314Z""#);
    }

    // ── MsDateTime round-trip tests ──

    #[test]
    fn ms_datetime_round_trip() {
        let json = r#""/Date(1573755038314)/""#;
        let dt: MsDateTime = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&dt).unwrap();
        assert_eq!(serialized, r#""2019-11-14T18:10:38.314Z""#);
    }

    #[test]
    fn ms_datetime_round_trip_zero() {
        let json = r#""/Date(0)/""#;
        let dt: MsDateTime = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&dt).unwrap();
        assert_eq!(serialized, r#""1970-01-01T00:00:00.000Z""#);
    }

    // ── Display tests ──

    #[test]
    fn ms_date_display() {
        let date = MsDate(NaiveDate::from_ymd_opt(2019, 10, 31).unwrap());
        assert_eq!(format!("{date}"), "2019-10-31");
    }

    #[test]
    fn ms_datetime_display() {
        let dt = MsDateTime(Utc.timestamp_opt(1_573_755_038, 314_000_000).unwrap());
        assert_eq!(format!("{dt}"), "2019-11-14T18:10:38.314Z");
    }

    // ── Error cases ──

    #[test]
    fn ms_date_invalid_format_error() {
        let json = r#""not-a-date""#;
        let result: Result<MsDate, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn ms_datetime_invalid_format_error() {
        let json = r#""not-a-datetime""#;
        let result: Result<MsDateTime, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ── Struct integration test ──

    #[test]
    fn ms_date_in_struct() {
        #[derive(Debug, Deserialize, Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct TestInvoice {
            date: MsDate,
            #[serde(rename = "UpdatedDateUTC")]
            updated_date_utc: MsDateTime,
        }

        let json =
            r#"{"Date": "/Date(1539993600000+0000)/", "UpdatedDateUTC": "/Date(1573755038314)/"}"#;
        let invoice: TestInvoice = serde_json::from_str(json).unwrap();
        assert_eq!(
            invoice.date.0,
            NaiveDate::from_ymd_opt(2018, 10, 20).unwrap()
        );
        assert_eq!(invoice.updated_date_utc.0.timestamp(), 1_573_755_038);

        // Serialize back
        let out = serde_json::to_string(&invoice).unwrap();
        assert!(out.contains("2018-10-20"));
        assert!(out.contains("2019-11-14T18:10:38.314Z"));
    }
}
