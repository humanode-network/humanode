//! The EIP-712 implementation and useful utilities.

use sha3::Digest;

mod domain;

pub use domain::Domain;

/// Provides the means to hash the type specification for EIP-712.
///
/// The `name` is the name of the type, the `fields` is a list of the field specifications
/// in the `<type> <name>` form.
fn hash_type(name: &'static str, fields: impl Iterator<Item = &'static str>) -> [u8; 32] {
    let mut hasher = sha3::Keccak256::new();

    hasher.update(name);

    hasher.update(b"(");

    let mut first = true;
    for field in fields {
        if first {
            first = false;
        } else {
            hasher.update(b",");
        }
        hasher.update(field);
    }

    hasher.update(b")");

    hasher.finalize().into()
}
