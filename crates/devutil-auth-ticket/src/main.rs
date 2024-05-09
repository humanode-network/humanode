//! The entrypoint to the auth ticket dev utils cli.

use devutil_auth_ticket::*;

/// Read the HEX-encoded binary value from an env var, and panic if we couldn't.
fn read_hex_env(key: &'static str) -> Vec<u8> {
    let val = std::env::var(key).unwrap();
    decode(val).unwrap()
}

fn main() {
    let robonode_secret_key = read_hex_env("ROBONODE_SECRET_KEY");
    let public_key = read_hex_env("AUTH_TICKET_PUBLIC_KEY");
    let authentication_nonce = read_hex_env("AUTH_TICKET_AUTHENTICATION_NONCE");

    let auth_ticket = AuthTicket {
        public_key,
        authentication_nonce,
    };

    let output = make(Input {
        robonode_secret_key,
        auth_ticket,
    })
    .unwrap();

    print!(
        "{}\n{}\n{}\n\n{:?}\n{:?}\n{:?}\n",
        encode(output.auth_ticket.clone()),
        encode(output.robonode_signature.clone()),
        encode(output.robonode_public_key.clone()),
        output.auth_ticket,
        output.robonode_signature,
        output.robonode_public_key,
    );
}
