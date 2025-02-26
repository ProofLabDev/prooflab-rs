use prooflab_io;

fn main() {
    // Read the input array
    let mut input: Vec<i32> = prooflab_io::read();

    // Commit the original array
    prooflab_io::commit(&input);

    // Bubble sort implementation
    let n = input.len();
    for i in 0..n {
        for j in 0..n - i - 1 {
            if input[j] > input[j + 1] {
                input.swap(j, j + 1);
            }
        }
    }

    // Commit the sorted array
    prooflab_io::commit(&input);
}

fn input() {
    // Example input array
    let numbers = vec![64, 34, 25, 12, 22, 11, 90];
    prooflab_io::write(&numbers);
}

fn output() {
    let (original, sorted): (Vec<i32>, Vec<i32>) = prooflab_io::out();

    println!("Original array: {:?}", original);
    println!("Sorted array:   {:?}", sorted);
}
