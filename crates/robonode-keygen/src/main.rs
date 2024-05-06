//! A tiny utility for generating a robonode key.
//! Prints the secret key in HEX.

use rand::rngs::OsRng;

fn main() {
    let mut csprng = OsRng {};
    let signing_key = robonode_crypto::SigningKey::generate(&mut csprng);
    println!("{}", hex::encode(signing_key.to_bytes()));
}
