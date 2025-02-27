#![no_main]
sp1_zkvm::entrypoint!(main);
use prooflab_io;
pub fn main() {
    println!("cycle-tracker-report-start: {}", env!("CARGO_PKG_NAME"));
    let n: u32 = sp1_zkvm::io::read();
    sp1_zkvm::io::commit(&n);

    let mut a: u32 = 0;
    let mut b: u32 = 1;
    for _ in 0..n {
        let mut c = a + b;
        c %= 7919; // Modulus to prevent overflow.
        a = b;
        b = c;
    }

    sp1_zkvm::io::commit(&a);
    sp1_zkvm::io::commit(&b);
    println!("cycle-tracker-report-end: {}", env!("CARGO_PKG_NAME"));
}
