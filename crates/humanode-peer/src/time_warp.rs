use sp_timestamp::Timestamp;

#[derive(Debug, Clone)]
pub struct TimeWarp {
    pub revive_timestamp: Timestamp,
    pub fork_timestamp: Timestamp,
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

pub fn current_timestamp() -> Timestamp {
    sp_timestamp::InherentDataProvider::from_system_time().timestamp()
}
