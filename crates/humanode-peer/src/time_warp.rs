//! Time warp mode logic.
//!
//! ## The issue's root cause.
//!
//! One invariant of BABE is that you have to produce at least one block every `epoch_duration` slots.
//! In case the network doesn't do it we get the following error during producing the block for
//! next epoch: "Import failed: Unexpected epoch change".
//!
//! ## Instruction to recover the network.
//!
//! 1. Extract the time of the last block that was finalized before the chain bricked (`FORK_TIMESTAMP`).
//!
//! 2. Revert the peer's data blocks to the latest finalized block by running `./humanode-peer revert`.
//!
//! 3. Define the time in the future when the warp is going to be started.
//!    In other words, it's `REVIVE_TIMESTAMP` (by default, timestamp of exact the peer binary run).
//!
//! 4. Define `WARP_FACTOR` that is going to be adopted to do time warp.
//!
//! 5. Run the peer in time warp mode by passing `--time-warp-fork-timestamp`, `--time-warp-revive-timestamp`,
//!    `--time-warp-factor`.
//!
//! 6. When the correct timestamp has been reached switch the peer to usual running mode.

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
        // u64 is big enough for this overflow to be practically impossible.
        let warped_timestamp =
            Timestamp::new(self.warp_factor.saturating_add(*self.fork_timestamp));

        let timestamp = if warped_timestamp < timestamp {
            tracing::debug!(target: "time-warp", message = format!("timestamp warped: {:?} to {:?} ({:?} since revival)",
                timestamp.as_millis(),
                warped_timestamp.as_millis(),
                time_since_revival)
            );

            warped_timestamp
        } else {
            tracing::debug!(target: "time-warp", message = format!("real timestamp has been reached: {:?}",
                timestamp.as_millis())
            );

            timestamp
        };

        sp_timestamp::InherentDataProvider::new(timestamp)
    }
}

/// Get current timestamp.
pub fn current_timestamp() -> Timestamp {
    sp_timestamp::InherentDataProvider::from_system_time().timestamp()
}
