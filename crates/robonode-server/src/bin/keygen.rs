use rand::rngs::OsRng;

fn main() {
    let mut csprng = OsRng {};
    let keypair = robonode_crypto::Keypair::generate(&mut csprng);
    println!("{}", hex::encode(keypair.to_bytes()));
}
