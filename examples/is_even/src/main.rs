use prooflab_io;

fn main() {
    let n: u32 = prooflab_io::read();
    prooflab_io::commit(&n);

    let is_even: bool = n % 2 == 0;

    prooflab_io::commit(&is_even);
}

fn input() {
    let n = 16u32;
    prooflab_io::write(&n);
}

fn output() {
    let (n, is_even): (u32, bool) = prooflab_io::out();

    println!("is_even: {}", is_even);
}
