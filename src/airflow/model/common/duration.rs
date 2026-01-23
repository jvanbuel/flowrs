use time::OffsetDateTime;

/// Trait for types that have start and end dates, enabling duration calculation.
pub trait TimeBounded {
    /// Returns the start date of the entity, if available.
    fn start_date(&self) -> Option<OffsetDateTime>;
    /// Returns the end date of the entity, if available.
    fn end_date(&self) -> Option<OffsetDateTime>;
}

/// Calculate duration in seconds for any time-bounded entity.
/// Returns None if `start_date` is not available.
/// For running entities (no `end_date`), uses current time.
pub fn calculate_duration<T: TimeBounded>(item: &T) -> Option<f64> {
    let start = item.start_date()?;
    let end = item.end_date().unwrap_or_else(OffsetDateTime::now_utc);
    Some((end - start).as_seconds_f64())
}

/// Format duration as human-readable string (e.g., "2h 30m", "45s", "1d 3h").
#[must_use]
pub fn format_duration(seconds: f64) -> String {
    if seconds < 60.0 {
        format!("{seconds:.0}s")
    } else if seconds < 3600.0 {
        let minutes = (seconds / 60.0).floor();
        let secs = (seconds % 60.0).floor();
        if secs > 0.0 {
            format!("{minutes:.0}m {secs:.0}s")
        } else {
            format!("{minutes:.0}m")
        }
    } else if seconds < 86400.0 {
        let hours = (seconds / 3600.0).floor();
        let minutes = ((seconds % 3600.0) / 60.0).floor();
        if minutes > 0.0 {
            format!("{hours:.0}h {minutes:.0}m")
        } else {
            format!("{hours:.0}h")
        }
    } else {
        let days = (seconds / 86400.0).floor();
        let hours = ((seconds % 86400.0) / 3600.0).floor();
        if hours > 0.0 {
            format!("{days:.0}d {hours:.0}h")
        } else {
            format!("{days:.0}d")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        // Seconds
        assert_eq!(format_duration(30.0), "30s");
        // Minutes
        assert_eq!(format_duration(90.0), "1m 30s");
        assert_eq!(format_duration(120.0), "2m");
        // Hours
        assert_eq!(format_duration(5400.0), "1h 30m");
        assert_eq!(format_duration(7200.0), "2h");
        // Days
        assert_eq!(format_duration(90000.0), "1d 1h");
        assert_eq!(format_duration(172_800.0), "2d");
    }
}
