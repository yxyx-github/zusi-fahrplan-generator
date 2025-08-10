#[allow(dead_code)]
pub mod with; // TODO: extract this module into a separate crate

use time::Duration;

pub fn parse_duration_from_time_string(time_str: &str) -> Result<Duration, String> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return Err("Invalid time format, expected HH:MM:SS".into());
    }
    let hours: i64 = parts[0].parse().map_err(|e| format!("Failed to parse hours: {e}"))?;
    let minutes: i64 = parts[1].parse().map_err(|e| format!("Failed to parse minutes: {e}"))?;
    let seconds: i64 = parts[2].parse().map_err(|e| format!("Failed to parse seconds: {e}"))?;
    Ok(Duration::hours(hours) + Duration::minutes(minutes) + Duration::seconds(seconds))
}

pub fn format_duration_as_time_string(duration: &Duration) -> String {
    let total_seconds = duration.whole_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_from_time_string() {
        assert_eq!(parse_duration_from_time_string("03:42:07").unwrap(), Duration::hours(3) + Duration::minutes(42) + Duration::seconds(7));
        assert_eq!(parse_duration_from_time_string("3:2:4").unwrap(), Duration::hours(3) + Duration::minutes(2) + Duration::seconds(4));
        assert_eq!(parse_duration_from_time_string("33:92:76").unwrap(), Duration::hours(33) + Duration::minutes(92) + Duration::seconds(76));
        assert_eq!(parse_duration_from_time_string("3A:9B:7C"), Err("Failed to parse hours: invalid digit found in string".into()));
        assert_eq!(parse_duration_from_time_string("3:9B:7C"), Err("Failed to parse minutes: invalid digit found in string".into()));
        assert_eq!(parse_duration_from_time_string("3:9:7C"), Err("Failed to parse seconds: invalid digit found in string".into()));
    }

    #[test]
    fn test_format_duration_as_time_string() {
        assert_eq!(format_duration_as_time_string(&(Duration::hours(3) + Duration::minutes(42) + Duration::seconds(7))), String::from("03:42:07"));
        assert_eq!(format_duration_as_time_string(&(Duration::hours(33) + Duration::minutes(92) + Duration::seconds(76))), String::from("34:33:16"));
    }
}