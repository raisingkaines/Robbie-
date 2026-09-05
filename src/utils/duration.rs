use chrono::{Duration, Utc, DateTime};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedDuration {
    Permanent,
    Temporary(Duration),
}

impl ParsedDuration {
    pub fn to_expiration_time(&self) -> Option<DateTime<Utc>> {
        match self {
            ParsedDuration::Permanent => None,
            ParsedDuration::Temporary(dur) => Some(Utc::now() + *dur),
        }
    }
}

impl fmt::Display for ParsedDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedDuration::Permanent => write!(f, "Permanent"),
            ParsedDuration::Temporary(dur) => {
                let total_secs = dur.num_seconds();
                if total_secs < 60 {
                    write!(f, "{} second(s)", total_secs)
                } else if total_secs < 3600 {
                    write!(f, "{} minute(s)", total_secs / 60)
                } else if total_secs < 86400 {
                    let hours = total_secs / 3600;
                    let mins = (total_secs % 3600) / 60;
                    if mins > 0 {
                        write!(f, "{}h {}m", hours, mins)
                    } else {
                        write!(f, "{} hour(s)", hours)
                    }
                } else {
                    let days = total_secs / 86400;
                    let hours = (total_secs % 86400) / 3600;
                    if hours > 0 {
                        write!(f, "{}d {}h", days, hours)
                    } else {
                        write!(f, "{} day(s)", days)
                    }
                }
            }
        }
    }
}

pub fn parse_duration_str(input: &str) -> Option<ParsedDuration> {
    let s = input.trim().to_lowercase();
    if s == "perm" || s == "permanent" || s == "0" || s == "forever" || s == "max" {
        return Some(ParsedDuration::Permanent);
    }

    if let Ok(mins) = s.parse::<i64>() {
        if mins <= 0 {
            return Some(ParsedDuration::Permanent);
        }
        return Some(ParsedDuration::Temporary(Duration::minutes(mins)));
    }

    let mut total_seconds: i64 = 0;
    let mut current_number = String::new();

    for ch in s.chars() {
        if ch.is_ascii_digit() {
            current_number.push(ch);
        } else {
            if current_number.is_empty() {
                continue;
            }
            let val: i64 = current_number.parse().ok()?;
            current_number.clear();

            match ch {
                's' => total_seconds += val,
                'm' => total_seconds += val * 60,
                'h' => total_seconds += val * 3600,
                'd' => total_seconds += val * 86400,
                'w' => total_seconds += val * 604800,
                'y' => total_seconds += val * 31536000,
                _ => return None,
            }
        }
    }

    if !current_number.is_empty() {
        if let Ok(mins) = current_number.parse::<i64>() {
            total_seconds += mins * 60;
        }
    }

    if total_seconds <= 0 {
        None
    } else {
        Some(ParsedDuration::Temporary(Duration::seconds(total_seconds)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_permanent_variants() {
        assert_eq!(parse_duration_str("perm"), Some(ParsedDuration::Permanent));
        assert_eq!(parse_duration_str("PERMANENT"), Some(ParsedDuration::Permanent));
        assert_eq!(parse_duration_str("0"), Some(ParsedDuration::Permanent));
        assert_eq!(parse_duration_str("forever"), Some(ParsedDuration::Permanent));
        assert_eq!(parse_duration_str("max"), Some(ParsedDuration::Permanent));
    }

    #[test]
    fn test_parse_numeric_minutes() {
        assert_eq!(
            parse_duration_str("30"),
            Some(ParsedDuration::Temporary(Duration::minutes(30)))
        );
        assert_eq!(parse_duration_str("-5"), Some(ParsedDuration::Permanent));
    }

    #[test]
    fn test_parse_compound_duration() {
        assert_eq!(
            parse_duration_str("1h30m"),
            Some(ParsedDuration::Temporary(Duration::seconds(5400)))
        );
        assert_eq!(
            parse_duration_str("2d"),
            Some(ParsedDuration::Temporary(Duration::days(2)))
        );
        assert_eq!(
            parse_duration_str("1w"),
            Some(ParsedDuration::Temporary(Duration::weeks(1)))
        );
    }

    #[test]
    fn test_parse_invalid() {
        assert_eq!(parse_duration_str("invalid"), None);
        assert_eq!(parse_duration_str("abc123x"), None);
    }

    #[test]
    fn test_display_formatting() {
        let perm = ParsedDuration::Permanent;
        assert_eq!(perm.to_string(), "Permanent");

        let thirty_secs = ParsedDuration::Temporary(Duration::seconds(30));
        assert_eq!(thirty_secs.to_string(), "30 second(s)");

        let ten_mins = ParsedDuration::Temporary(Duration::minutes(10));
        assert_eq!(ten_mins.to_string(), "10 minute(s)");

        let hours_mins = ParsedDuration::Temporary(Duration::minutes(90));
        assert_eq!(hours_mins.to_string(), "1h 30m");

        let two_days = ParsedDuration::Temporary(Duration::days(2));
        assert_eq!(two_days.to_string(), "2 day(s)");
    }
}

