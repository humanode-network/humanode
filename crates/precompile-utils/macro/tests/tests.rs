use sha3::{Digest, Keccak256};

#[precompile_utils_macro::generate_function_selector]
pub enum Action {
    Toto = "toto()",
    Tata = "tata()",
}

#[test]
fn test_keccak256() {
    assert_eq!(
        &precompile_utils_macro::keccak256!(""),
        Keccak256::digest(b"").as_slice(),
    );
    assert_eq!(
        &precompile_utils_macro::keccak256!("toto()"),
        Keccak256::digest(b"toto()").as_slice(),
    );
    assert_ne!(
        &precompile_utils_macro::keccak256!("toto()"),
        Keccak256::digest(b"tata()").as_slice(),
    );
}

#[test]
fn test_generate_function_selector() {
    assert_eq!(
        &(Action::Toto as u32).to_be_bytes()[..],
        &Keccak256::digest(b"toto()")[0..4],
    );
    assert_eq!(
        &(Action::Tata as u32).to_be_bytes()[..],
        &Keccak256::digest(b"tata()")[0..4],
    );
    assert_ne!(Action::Toto as u32, Action::Tata as u32);
}
