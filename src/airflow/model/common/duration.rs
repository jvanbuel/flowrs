use time::OffsetDateTime;

/// Trait for types that have start and end dates, enabling duration calculation.
pub trait TimeBounded {
    /// Returns the start date of the entity, if available.
    fn start_date(&self) -> Option<OffsetDateTime>;
    /// Returns the end date of the entity, if available.
    fn end_date(&self) -> Option<OffsetDateTime>;
    /// Returns whether the entity is currently running/active.
    /// Used to decide if current time should be used as end date for duration calculation.
    fn is_running(&self) -> bool;
}

/// Calculate duration in seconds for any time-bounded entity.
/// Returns None if `start_date` is not available or if `end_date` precedes `start_date`.
/// For running entities (no `end_date`), uses current time.
/// For non-running entities without `end_date`, returns None.
pub fn calculate_duration<T: TimeBounded>(item: &T) -> Option<f64> {
    let start = item.start_date()?;
    let end = match item.end_date() {
        Some(end) => end,
        None if item.is_running() => OffsetDateTime::now_utc(),
        None => return None,
    };
    if end < start {
        return None;
    }
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
    use time::Duration;

    struct TestEntity {
        start: Option<OffsetDateTime>,
        end: Option<OffsetDateTime>,
        running: bool,
    }

    impl TimeBounded for TestEntity {
        fn start_date(&self) -> Option<OffsetDateTime> {
            self.start
        }
        fn end_date(&self) -> Option<OffsetDateTime> {
            self.end
        }
        fn is_running(&self) -> bool {
            self.running
        }
    }

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

    #[test]
    fn test_calculate_duration_with_start_and_end() {
        let now = OffsetDateTime::now_utc();
        let entity = TestEntity {
            start: Some(now - Duration::seconds(120)),
            end: Some(now),
            running: false,
        };
        let duration = calculate_duration(&entity).unwrap();
        assert!((duration - 120.0).abs() < 1.0);
    }

    #[test]
    fn test_calculate_duration_no_start_returns_none() {
        let entity = TestEntity {
            start: None,
            end: Some(OffsetDateTime::now_utc()),
            running: false,
        };
        assert!(calculate_duration(&entity).is_none());
    }

    #[test]
    fn test_calculate_duration_running_no_end_uses_now() {
        let entity = TestEntity {
            start: Some(OffsetDateTime::now_utc() - Duration::seconds(60)),
            end: None,
            running: true,
        };
        let duration = calculate_duration(&entity).unwrap();
        assert!(duration >= 59.0 && duration <= 62.0);
    }

    #[test]
    fn test_calculate_duration_not_running_no_end_returns_none() {
        let entity = TestEntity {
            start: Some(OffsetDateTime::now_utc() - Duration::seconds(60)),
            end: None,
            running: false,
        };
        assert!(calculate_duration(&entity).is_none());
    }

    #[test]
    fn test_calculate_duration_end_before_start_returns_none() {
        let now = OffsetDateTime::now_utc();
        let entity = TestEntity {
            start: Some(now),
            end: Some(now - Duration::seconds(60)),
            running: false,
        };
        assert!(calculate_duration(&entity).is_none());
    }
}
