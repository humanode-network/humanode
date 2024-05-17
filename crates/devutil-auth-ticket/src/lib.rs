//! Auth Ticket development utilities.

pub use hex::{decode, encode};
pub use primitives_auth_ticket::{AuthTicket, OpaqueAuthTicket};
use robonode_crypto::ed25519_dalek::Signer;

/// The input required to generate an auth ticket.
pub struct Input {
    /// The robonode secret key to use for authticket response signing.
    pub robonode_secret_key: Vec<u8>,
    /// The auth ticket to sign.
    pub auth_ticket: AuthTicket,
}

/// The output produced from this utility.
pub struct Output {
    /// The opaque auth ticket blob.
    pub auth_ticket: Vec<u8>,
    /// The robonode signature.
    pub robonode_signature: Vec<u8>,
    /// The public key from the robonode keypair.
    pub robonode_public_key: Vec<u8>,
}

/// Run the auth ticket devutil logic, producing an output for a given input.
pub fn make(input: Input) -> Result<Output, anyhow::Error> {
    let Input {
        auth_ticket,
        robonode_secret_key,
    } = input;

    let mut robonode_secret_key_buf = robonode_crypto::SecretKey::default();
    robonode_secret_key_buf.copy_from_slice(&robonode_secret_key);

    let robonode_singing_key = robonode_crypto::SigningKey::from_bytes(&robonode_secret_key_buf);

    let opaque_auth_ticket = OpaqueAuthTicket::from(&auth_ticket);

    let robonode_signature = robonode_singing_key
        .sign(opaque_auth_ticket.as_ref())
        .to_bytes();

    assert!(robonode_singing_key
        .verify(
            opaque_auth_ticket.as_ref(),
            &robonode_crypto::Signature::try_from(&robonode_signature[..]).unwrap()
        )
        .is_ok());

    Ok(Output {
        auth_ticket: opaque_auth_ticket.into(),
        robonode_signature: robonode_signature.into(),
        robonode_public_key: robonode_singing_key.verifying_key().as_bytes().to_vec(),
    })
}
