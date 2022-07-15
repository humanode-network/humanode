use super::*;

pub fn public(secret: &libsecp256k1::SecretKey) -> libsecp256k1::PublicKey {
    libsecp256k1::PublicKey::from_secret_key(secret)
}
pub fn eth(secret: &libsecp256k1::SecretKey) -> EthereumAddress {
    let mut res = EthereumAddress::default();
    res.0
        .copy_from_slice(&keccak_256(&public(secret).serialize()[1..65])[12..]);
    res
}
pub fn sig<T: Config>(
    secret: &libsecp256k1::SecretKey,
    what: &[u8],
    extra: &[u8],
) -> EcdsaSignature {
    let msg = keccak_256(&<super::Pallet<T>>::ethereum_signable_message(
        &to_ascii_hex(what)[..],
        extra,
    ));
    let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&msg), secret);
    let mut r = [0u8; 65];
    r[0..64].copy_from_slice(&sig.serialize()[..]);
    r[64] = recovery_id.serialize();
    EcdsaSignature(r)
}

pub fn alice() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}
pub fn bob() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
}
pub fn dave() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Dave")).unwrap()
}
pub fn eve() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Eve")).unwrap()
}
pub fn frank() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Frank")).unwrap()
}
