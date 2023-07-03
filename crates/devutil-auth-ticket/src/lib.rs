//! Auth Ticket development utilities.

pub use hex::{decode, encode};
pub use primitives_auth_ticket::{AuthTicket, OpaqueAuthTicket};
use robonode_crypto::{ed25519_dalek::Signer, Keypair};

/// The input required to generate an auth ticket.
pub struct Input {
    /// The robonode keypair to use for authticket reponse signing.
    pub robonode_keypair: Vec<u8>,
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
        robonode_keypair,
    } = input;

    let mut robonode_keypair_buf = [0u8; 64];
    robonode_keypair_buf.copy_from_slice(&robonode_keypair);

    let robonode_keypair = Keypair::from_keypair_bytes(&robonode_keypair_buf)?;

    let opaque_auth_ticket = OpaqueAuthTicket::from(&auth_ticket);

    let robonode_signature = robonode_keypair
        .sign(opaque_auth_ticket.as_ref())
        .to_bytes();

    assert!(robonode_keypair
        .verify(
            opaque_auth_ticket.as_ref(),
            &robonode_crypto::Signature::try_from(&robonode_signature[..]).unwrap()
        )
        .is_ok());

    Ok(Output {
        auth_ticket: opaque_auth_ticket.into(),
        robonode_signature: robonode_signature.into(),
        robonode_public_key: robonode_keypair.as_ref().as_bytes()[..].into(),
    })
}
