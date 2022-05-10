//! DisplayMoment implementation to display timestamp.

use chrono::{TimeZone, Utc};

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
        let dt = Utc.timestamp_millis(self.0 as i64);
        write!(f, "{}", dt)
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
