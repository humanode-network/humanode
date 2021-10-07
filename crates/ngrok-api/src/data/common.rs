//! Common types used in both requests and responses.

use serde::{Deserialize, Serialize};

/// One of the supported protocols, or something else.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    /// The HTTP protocol.
    Http,
    /// The HTTPS protocol.
    Https,
    /// The TCP protocol.
    Tcp,
    /// The TLS protocol.
    Tls,
    /// Some other protocol.
    Other(String),
}
