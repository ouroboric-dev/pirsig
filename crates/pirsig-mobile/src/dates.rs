//! Dates as the design writes them: "18 Apr 2027", "15 Dec" in the current
//! year, "Today", and "2h ago" for when a note changed. Pure, so they are
//! tested natively; the app passes in the time from the browser.

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const MONTHS_LONG: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

/// A calendar day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Day {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Day {
    /// Read the start of an ISO 8601 date or timestamp: `2027-04-18`, `2026-10-29T09:41:00+00:00`.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let date = s.get(..10)?;
        let mut it = date.split('-');
        let year = it.next()?.parse().ok()?;
        let month = it.next()?.parse().ok().filter(|m| (1..=12).contains(m))?;
        let day = it.next()?.parse().ok().filter(|d| (1..=31).contains(d))?;
        Some(Self { year, month, day })
    }

    /// Days since 1970-01-01 (Howard Hinnant's `days_from_civil`).
    #[must_use]
    pub fn ordinal(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    fn month_name(self) -> &'static str {
        MONTHS[(self.month - 1) as usize]
    }

    /// "Thursday, 29 October".
    #[must_use]
    pub fn long(self) -> String {
        let weekday = WEEKDAYS[usize::try_from((self.ordinal() + 3).rem_euclid(7)).unwrap_or(0)];
        format!(
            "{weekday}, {} {}",
            self.day,
            MONTHS_LONG[(self.month - 1) as usize]
        )
    }

    /// "18 April 2027".
    #[must_use]
    pub fn full(self) -> String {
        format!(
            "{} {} {}",
            self.day,
            MONTHS_LONG[(self.month - 1) as usize],
            self.year
        )
    }
}

/// A due date in a list: "Today", "15 Dec" this year, "18 Apr 2027" another.
#[must_use]
pub fn due(date: &str, today: Day) -> String {
    let Some(d) = Day::parse(date) else {
        return date.to_string();
    };
    if d == today {
        "Today".into()
    } else if d.year == today.year {
        format!("{} {}", d.day, d.month_name())
    } else {
        format!("{} {} {}", d.day, d.month_name(), d.year)
    }
}

/// As [`due`], but a date within a year of `today` drops its year: "10 Apr".
#[must_use]
pub fn due_short(date: &str, today: Day) -> String {
    match (Day::parse(date), days_until(date, today)) {
        (Some(d), Some(n)) if d != today && n.abs() < 365 => {
            format!("{} {}", d.day, d.month_name())
        }
        _ => due(date, today),
    }
}

/// Days from `today` until `date`; negative when it has passed.
#[must_use]
pub fn days_until(date: &str, today: Day) -> Option<i64> {
    Day::parse(date).map(|d| d.ordinal() - today.ordinal())
}

/// When something changed, from seconds since the epoch: "just now", "2h ago",
/// "today", "yesterday", "3 days ago", else the date.
#[must_use]
pub fn ago(timestamp: &str, now_secs: i64, today: Day) -> String {
    let Some(d) = Day::parse(timestamp) else {
        return String::new();
    };
    let secs = parse_seconds(timestamp);
    let days = today.ordinal() - d.ordinal();
    match (secs.map(|s| now_secs - s), days) {
        (Some(s), _) if (0..60).contains(&s) => "just now".into(),
        (Some(s), _) if (0..3600).contains(&s) => format!("{}m ago", s / 60),
        (Some(s), _) if (0..6 * 3600).contains(&s) => format!("{}h ago", s / 3600),
        (_, 0) => "today".into(),
        (_, 1) => "yesterday".into(),
        (_, 2..=6) => format!("{days} days ago"),
        _ => due(timestamp, today),
    }
}

/// Seconds since the epoch of an RFC 3339 timestamp, offset applied.
#[must_use]
pub fn parse_seconds(s: &str) -> Option<i64> {
    let day = Day::parse(s)?;
    let time = s.get(11..19)?;
    let mut hms = time.split(':').map(str::parse::<i64>);
    let (h, m, sec) = (hms.next()?.ok()?, hms.next()?.ok()?, hms.next()?.ok()?);
    let rest = &s[19..];
    let rest = rest.trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
    let offset = match rest.as_bytes().first() {
        Some(b'+' | b'-') => {
            let sign = if rest.starts_with('-') { -1 } else { 1 };
            let oh: i64 = rest.get(1..3)?.parse().ok()?;
            let om: i64 = rest.get(4..6)?.parse().ok()?;
            sign * (oh * 3600 + om * 60)
        }
        _ => 0,
    };
    Some(day.ordinal() * 86_400 + h * 3600 + m * 60 + sec - offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TODAY: Day = Day {
        year: 2026,
        month: 10,
        day: 29,
    };

    #[test]
    fn due_dates_as_the_design_writes_them() {
        assert_eq!(due("2026-10-29", TODAY), "Today");
        assert_eq!(due("2026-12-15", TODAY), "15 Dec");
        assert_eq!(due("2027-04-18", TODAY), "18 Apr 2027");
        assert_eq!(due_short("2027-04-10", TODAY), "10 Apr");
        assert_eq!(due_short("2026-10-29", TODAY), "Today");
        assert_eq!(due_short("2028-04-10", TODAY), "10 Apr 2028");
    }

    #[test]
    fn long_and_full_dates() {
        assert_eq!(TODAY.long(), "Thursday, 29 October");
        assert_eq!(Day::parse("2027-04-18").unwrap().full(), "18 April 2027");
        assert_eq!(days_until("2027-04-18", TODAY), Some(171));
    }

    #[test]
    fn ago_steps_down_from_minutes_to_dates() {
        let now = parse_seconds("2026-10-29T12:00:00+00:00").unwrap();
        assert_eq!(ago("2026-10-29T11:59:30+00:00", now, TODAY), "just now");
        assert_eq!(ago("2026-10-29T10:00:00+00:00", now, TODAY), "2h ago");
        assert_eq!(ago("2026-10-29T01:00:00+00:00", now, TODAY), "today");
        assert_eq!(ago("2026-10-28T09:00:00+00:00", now, TODAY), "yesterday");
        assert_eq!(ago("2026-10-26T09:00:00+00:00", now, TODAY), "3 days ago");
        assert_eq!(ago("2026-03-01T09:00:00+00:00", now, TODAY), "1 Mar");
    }

    #[test]
    fn offsets_and_fractions() {
        assert_eq!(
            parse_seconds("2026-10-29T12:00:00.123456-03:00"),
            parse_seconds("2026-10-29T15:00:00Z")
        );
        assert_eq!(Day::parse("2026-13-01"), None);
    }
}
