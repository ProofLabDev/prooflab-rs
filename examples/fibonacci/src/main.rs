use prooflab_io;

fn main() {
    let n: u32 = prooflab_io::read();
    prooflab_io::commit(&n);

    let mut a: u32 = 0;
    let mut b: u32 = 1;
    for _ in 0..n {
        let mut c = a + b;
        c %= 7919; // Modulus to prevent overflow.
        a = b;
        b = c;
    }

    prooflab_io::commit(&a);
    prooflab_io::commit(&b);
}

fn input() {
    let n = std::env::var("BENCHMARK_SIZE")
        .ok()
        .and_then(|val| val.parse::<u32>().ok())
        .unwrap_or(1000u32);
    prooflab_io::write(&n);
}

fn output() {
    let (n, a, b): (u32, u32, u32) = prooflab_io::out();

    println!("n: {}", n);
    println!("a: {}", a);
    println!("b: {}", b);
}
