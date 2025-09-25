use prooflab_io;
use sha3::{Digest, Sha3_256};

fn main() {
    let data: Vec<u8> = prooflab_io::read();
    let digest: [u8; 32] = Sha3_256::digest(&data).into();
    prooflab_io::commit(&data);
    prooflab_io::commit(&digest);
}

fn input() {
    let repeat_count = std::env::var("BENCHMARK_SIZE")
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(1);

    let base_message = b"Hello, SHA3!";
    let message: Vec<u8> = std::iter::repeat(base_message)
        .take(repeat_count)
        .flat_map(|chunk| chunk.iter().copied())
        .collect();

    prooflab_io::write(&message);
}

fn output() {
    let (data, digest): (Vec<u8>, [u8; 32]) = prooflab_io::out();
    println!("Input bytes: {} bytes", data.len());
    println!("SHA3-256 digest: {:?}", digest);
}
