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
    // Get benchmark size or default to a small array
    let size = std::env::var("BENCHMARK_SIZE")
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(7);
    
    // Generate an array of the specified size
    // For deterministic results, we'll use a pattern that scales with size
    let mut numbers = Vec::with_capacity(size);
    
    // Fill with descending values for worst-case sorting scenario
    for i in 0..size {
        numbers.push((size - i) as i32);
    }
    
    prooflab_io::write(&numbers);
}

fn output() {
    let (original, sorted): (Vec<i32>, Vec<i32>) = prooflab_io::out();

    println!("Original array: {:?}", original);
    println!("Sorted array:   {:?}", sorted);
}
