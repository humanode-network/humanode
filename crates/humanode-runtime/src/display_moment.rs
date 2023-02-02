//! `DisplayMoment` implementation to display timestamp.

use chrono::{LocalResult, TimeZone, Utc};

use crate::UnixMilliseconds;

/// Provides a functionality to extract and display timestamp properly.
pub struct DisplayMoment(UnixMilliseconds);

impl From<UnixMilliseconds> for DisplayMoment {
    fn from(moment: UnixMilliseconds) -> Self {
        DisplayMoment(moment)
    }
}

impl core::fmt::Display for DisplayMoment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // i64 is big enough to process `UnixMilliseconds` properly.
        match Utc.timestamp_millis_opt(self.0.try_into().unwrap()) {
            LocalResult::None => write!(f, "[invalid milliseconds {}]", self.0),
            LocalResult::Single(dt) => write!(f, "{dt}"),
            LocalResult::Ambiguous(_, _) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_display_output() {
        // https://www.epochconverter.com has been used for getting test input data.
        assert_eq!(
            DisplayMoment::from(1637521860001).to_string(),
            "2021-11-21 19:11:00.001 UTC"
        );
    }
}
