//! Response types.

use serde::{Deserialize, Serialize};

use super::common::Protocol;

/// The response envelope.
#[derive(Debug, Serialize, Deserialize)]
pub struct Envelope<Payload> {
    /// The request `uri`.
    pub uri: String,
    /// The response payload.
    #[serde(flatten)]
    pub payload: Payload,
}

/// The tunnels list.
#[derive(Debug, Serialize, Deserialize)]
pub struct TunnelsList {
    /// A list of tunnels.
    pub tunnels: Vec<Tunnel>,
}

/// The tunnel resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct Tunnel {
    /// Tunnel name.
    pub name: String,
    /// The public URL of the tunnel.
    pub public_url: String,
    /// The protocol.
    pub proto: Protocol,
    /// The configuration of the tunnel.
    pub config: TunnelConfig,
}

/// Configuration of the tunnel.
#[derive(Debug, Serialize, Deserialize)]
pub struct TunnelConfig {
    /// The port to tunnel.
    pub addr: String,
    /// Whether to inspect the tunnel or not.
    pub inspect: bool,
}
