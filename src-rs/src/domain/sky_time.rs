use chrono::{
    DateTime, Datelike, Duration, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc,
};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

pub const SKY_TIME_ZONE: Tz = chrono_tz::America::Los_Angeles;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl SkyDate {
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        NaiveDate::from_ymd_opt(year, month, day).map(|_| Self { year, month, day })
    }

    pub fn from_naive(date: NaiveDate) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .ok()
            .map(Self::from_naive)
    }

    pub fn to_naive(self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, self.day)
            .expect("SkyDate is constructed from a valid calendar date")
    }

    pub fn add_days(self, days: i64) -> Self {
        Self::from_naive(self.to_naive() + Duration::days(days))
    }

    pub fn day_of_month(self) -> u32 {
        self.day
    }

    pub fn day_of_week_temporal(self) -> u32 {
        self.to_naive().weekday().number_from_monday()
    }
}

impl std::fmt::Display for SkyDate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

pub fn sky_date_for_instant(instant: DateTime<Utc>) -> SkyDate {
    SkyDate::from_naive(instant.with_timezone(&SKY_TIME_ZONE).date_naive())
}

pub fn sky_wall_time_to_instant(date: SkyDate, hour: u32, minute: u32) -> Option<DateTime<Utc>> {
    let time = NaiveTime::from_hms_opt(hour, minute, 0)?;
    let local = NaiveDateTime::new(date.to_naive(), time);

    match SKY_TIME_ZONE.from_local_datetime(&local) {
        LocalResult::Single(value) => Some(value.with_timezone(&Utc)),
        LocalResult::Ambiguous(_, _) | LocalResult::None => None,
    }
}

pub fn format_sky_time(instant: DateTime<Utc>) -> String {
    let sky = instant.with_timezone(&SKY_TIME_ZONE);
    sky.format("%-I:%M %p %Z").to_string()
}

pub fn format_local_time(
    instant: DateTime<Utc>,
    hour_cycle: &str,
    time_zone: Option<&str>,
) -> String {
    let zone = time_zone
        .and_then(|value| value.parse::<Tz>().ok())
        .unwrap_or(chrono_tz::UTC);
    let local = instant.with_timezone(&zone);

    match hour_cycle {
        "24h" => local.format("%H:%M").to_string(),
        _ => local.format("%-I:%M %p").to_string(),
    }
}

pub fn format_local_date_time(
    instant: DateTime<Utc>,
    hour_cycle: &str,
    time_zone: Option<&str>,
) -> String {
    let zone = time_zone
        .and_then(|value| value.parse::<Tz>().ok())
        .unwrap_or(chrono_tz::UTC);
    let local = instant.with_timezone(&zone);

    match hour_cycle {
        "24h" => local.format("%H:%M:%S %Z").to_string(),
        _ => local.format("%-I:%M:%S %p %Z").to_string(),
    }
}

pub fn format_duration(milliseconds: i64) -> String {
    let total_seconds = std::cmp::max(0, (milliseconds + 999) / 1000);
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        return format!("{hours}h {minutes:02}m");
    }

    if minutes > 0 {
        return format!("{minutes}m {seconds:02}s");
    }

    format!("{seconds}s")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn january_daily_reset_uses_pst() {
        let date = SkyDate::parse("2026-01-15").unwrap();
        let instant = sky_wall_time_to_instant(date, 0, 0).unwrap();

        assert_eq!(
            instant.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-01-15T08:00:00Z"
        );
    }

    #[test]
    fn july_daily_reset_uses_pdt() {
        let date = SkyDate::parse("2026-07-15").unwrap();
        let instant = sky_wall_time_to_instant(date, 0, 0).unwrap();

        assert_eq!(
            instant.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-07-15T07:00:00Z"
        );
    }

    #[test]
    fn spring_forward_nonexistent_sky_local_times_are_skipped() {
        let date = SkyDate::parse("2026-03-08").unwrap();

        assert!(sky_wall_time_to_instant(date, 2, 0).is_none());
    }

    #[test]
    fn formats_duration_like_the_frontend_helper() {
        assert_eq!(format_duration(4_999), "5s");
        assert_eq!(format_duration(65_001), "1m 06s");
        assert_eq!(format_duration(3_601_000), "1h 00m");
    }
}
