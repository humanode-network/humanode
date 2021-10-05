//! Handy utils for writing CLI related code.

/// The wrapper for easier handling of the application errors.
pub fn application_error<E>(err: E) -> sc_cli::Error
where
    E: ToOwned,
    E::Owned: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
{
    // Clippy is straight up stupid about this one. We obviously need to do `to_owned` here, since
    // we're explicitly into the borrow mechanics as it should've been able to tell from
    // the generic's trait bounds. In this case, `to_owned` is not necessarilty a `clone`.
    #[allow(clippy::redundant_clone)]
    sc_cli::Error::Application(err.to_owned().into())
}
