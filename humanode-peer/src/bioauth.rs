//! Biometric authentication flow.

/// The pair of keys used in asymmetric cryptography.
type KeyPair = (); // TODO

/// Execute the bioauth routine.
pub async fn run() -> Result<KeyPair, anyhow::Error> {
    Ok(())
}
