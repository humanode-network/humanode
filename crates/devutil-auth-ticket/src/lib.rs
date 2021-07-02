use std::convert::TryFrom;

pub use hex::{decode, encode};
pub use primitives_auth_ticket::{AuthTicket, OpaqueAuthTicket};

use robonode_crypto::{ed25519_dalek::Signer, Keypair};

pub struct Input {
    pub robonode_keypair: Vec<u8>,
    pub auth_ticket: AuthTicket,
}

pub struct Output {
    pub auth_ticket: Vec<u8>,
    pub robonode_signature: Vec<u8>,
    pub robonode_public_key: Vec<u8>,
}

pub fn make(input: Input) -> Result<Output, anyhow::Error> {
    let Input {
        auth_ticket,
        robonode_keypair,
    } = input;

    let robonode_keypair = Keypair::from_bytes(&robonode_keypair)?;

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
        robonode_public_key: robonode_keypair.public.as_bytes()[..].into(),
    })
}
