//! Humanode peer configuration.

/// Peer configuration, including both standard substrate configuration and our custom extensions.
pub struct Configuration {
    /// Standard substrate configuration.
    pub substrate: sc_service::Configuration,
}
