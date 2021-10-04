//! Request types.

use serde::{Deserialize, Serialize};

use super::common::Protocol;

/// A request to list the tunnels.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListTunnels;

/// A request to start a new tunnel.
#[derive(Debug, Serialize, Deserialize)]
pub struct StartTunnel {
    /// The name of the tunnel.
    pub name: String,
    /// The protocol to tunnel.
    pub proto: Protocol,
    /// The port to tunnel.
    pub addr: String,
}

/// A request to get a tunnel info.
#[derive(Debug, Serialize, Deserialize)]
pub struct TunnelInfo;

/// A request to stop a tunnel.
#[derive(Debug, Serialize, Deserialize)]
pub struct StopTunnel;
