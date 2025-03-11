use sha3::{Digest as _, Keccak256};

fn main() {
    let data: Vec<u8> = prooflab_io::read();
    let hash: [u8; 32] = Keccak256::digest(&data).into();
    prooflab_io::commit(&hash);
}

fn input() {
    let repeat_count = std::env::var("BENCHMARK_SIZE")
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(1);

    let base_message = b"Hello, world!";
    let message: Vec<u8> = std::iter::repeat(base_message)
        .take(repeat_count)
        .flat_map(|chunk| chunk.iter().copied())
        .collect();

    prooflab_io::write(&message);
}

fn output() {
    let hash: [u8; 32] = prooflab_io::out();
    println!("Keccak256 hash: {:?}", hash);
}
