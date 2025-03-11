//For acceleration we require the user defines the respective crate import since they are specific and needed to compile
use prooflab_io;
use sha2::{Digest, Sha256};

fn main() {
    let data: String = prooflab_io::read();
    let digest = Sha256::digest(&data.as_bytes());
    let digest_array: [u8; 32] = digest.into();
    prooflab_io::commit(&data);
    prooflab_io::commit(&digest_array);
}

fn input() {
    let repeat_count = std::env::var("BENCHMARK_SIZE")
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(1);

    let base_string = "RISCV IS COOL!!!";
    let data = std::iter::repeat(base_string)
        .take(repeat_count)
        .collect::<String>();

    prooflab_io::write(&data);
}

fn output() {
    let (data, digest): (String, [u8; 32]) = prooflab_io::out();
    println!("Input data: {}", data);
    println!("SHA256 digest: {:?}", digest);
}
