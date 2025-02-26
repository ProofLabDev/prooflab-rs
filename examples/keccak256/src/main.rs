use sha3::{Digest as _, Keccak256};

fn main() {
    let data: Vec<u8> = prooflab_io::read();
    let hash: [u8; 32] = Keccak256::digest(&data).into();
    prooflab_io::commit(&hash);
}

fn input() {
    let message = b"Hello, world!".to_vec();
    prooflab_io::write(&message);
}

fn output() {
    let hash: [u8; 32] = prooflab_io::out();
    println!("Keccak256 hash: {:?}", hash);
}
