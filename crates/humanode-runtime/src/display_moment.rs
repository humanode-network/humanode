//! DisplayMoment implementation to display timestamp.

use chrono::{DateTime, TimeZone, Utc};

/// Provides a functionality to extract and display timestamp properly.
pub struct DisplayMoment(DateTime<Utc>);

impl From<u64> for DisplayMoment {
    fn from(moment: u64) -> Self {
        let dt = Utc.timestamp(moment as i64, 0);
        DisplayMoment(dt)
    }
}

impl core::fmt::Display for DisplayMoment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_display_output() {
        // https://www.epochconverter.com has been used for getting test input data.
        assert_eq!(
            DisplayMoment::from(1637497887).to_string(),
            "2021-11-21 12:31:27 UTC"
        );
    }
}
