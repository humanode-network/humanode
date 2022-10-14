//! Time warp mode logic.

use sp_timestamp::Timestamp;

/// Reasonable default warp factor for 6s block time production.
pub const DEFAULT_WARP_FACTOR: u64 = 12;

/// Time warp mode to simulate time acceleration.
#[derive(Debug, Clone)]
pub struct TimeWarp {
    /// The time in the future when the warp is going to be started.
    pub revive_timestamp: Timestamp,
    /// The time of the last block that was finalized before the chain bricked.
    pub fork_timestamp: Timestamp,
    /// Warp factor that is going to be adopted.
    pub warp_factor: u64,
}

impl TimeWarp {
    /// Apply time warp.
    pub fn apply_time_warp(&self, timestamp: Timestamp) -> sp_timestamp::InherentDataProvider {
        let time_since_revival = timestamp.saturating_sub(self.revive_timestamp.into());
        let warped_timestamp = self.fork_timestamp + self.warp_factor * time_since_revival;

        tracing::debug!(target: "time-warp", message = format!("timestamp warped: {:?} to {:?} ({:?} since revival)",
            timestamp.as_millis(),
            warped_timestamp.as_millis(),
            time_since_revival)
        );

        let timestamp = timestamp.min(warped_timestamp);

        sp_timestamp::InherentDataProvider::new(timestamp)
    }
}

/// Get current timestamp.
pub fn current_timestamp() -> Timestamp {
    sp_timestamp::InherentDataProvider::from_system_time().timestamp()
}
