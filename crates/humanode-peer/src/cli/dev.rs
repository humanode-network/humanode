//! Development related cli utils.

/// Available block import sealing methods.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum DevBlockImportSealing {
    /// Seal using rpc method.
    Manual,
    /// Seal when transaction is executed.
    Instant,
}
