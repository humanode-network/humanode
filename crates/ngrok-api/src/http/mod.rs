//! `http` crate integration.

use std::borrow::Cow;

use crate::data::{request, response};

/// The definition of the HTTP response, bound to the request body.
pub trait Definition: serde::Serialize {
    /// The request method.
    const METHOD: http::Method;

    /// The request URL params.
    type Params;

    /// The response type.
    type Response: for<'de> serde::Deserialize<'de>;

    /// The request path segments.
    fn path_segments<'a>(params: Self::Params) -> Box<dyn Iterator<Item = Cow<'static, str>> + 'a>;
}

impl Definition for request::ListTunnels {
    const METHOD: http::Method = http::Method::GET;
    type Params = ();
    type Response = response::TunnelsList;

    fn path_segments<'a>(
        _params: Self::Params,
    ) -> Box<dyn Iterator<Item = Cow<'static, str>> + 'a> {
        tunnels()
    }
}

impl Definition for request::StartTunnel {
    const METHOD: http::Method = http::Method::POST;
    type Params = ();
    type Response = response::Tunnel;

    fn path_segments<'a>(
        _params: Self::Params,
    ) -> Box<dyn Iterator<Item = Cow<'static, str>> + 'a> {
        tunnels()
    }
}

impl Definition for request::TunnelInfo {
    const METHOD: http::Method = http::Method::GET;
    type Params = (Cow<'static, str>,);
    type Response = response::Tunnel;

    fn path_segments<'a>(params: Self::Params) -> Box<dyn Iterator<Item = Cow<'static, str>> + 'a> {
        named_tunnel(params.0)
    }
}

impl Definition for request::StopTunnel {
    const METHOD: http::Method = http::Method::DELETE;
    type Params = (Cow<'static, str>,);
    type Response = ();

    fn path_segments<'a>(params: Self::Params) -> Box<dyn Iterator<Item = Cow<'static, str>> + 'a> {
        named_tunnel(params.0)
    }
}

/// Produce "tunnels" segment.
fn tunnels() -> Box<dyn Iterator<Item = Cow<'static, str>>> {
    Box::new(std::iter::once(Cow::Borrowed("tunnels")))
}

/// Produce "tunnels" segment and the tunnel name.
fn named_tunnel(name: Cow<'static, str>) -> Box<dyn Iterator<Item = Cow<'static, str>>> {
    Box::new(std::iter::once(Cow::Borrowed("tunnels")).chain(std::iter::once(name)))
}
