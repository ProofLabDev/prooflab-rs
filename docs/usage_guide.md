# ProofLab Usage Guide

This comprehensive guide explains how to use ProofLab to develop zero-knowledge applications using zkVMs like SP1 and RISC0.

## Table of Contents

1. [Understanding the Architecture](#understanding-the-architecture)
2. [Project Structure](#project-structure)
3. [Creating a New Project](#creating-a-new-project)
4. [The Three Core Functions](#the-three-core-functions)
5. [Data Flow Between Host and Guest](#data-flow-between-host-and-guest)
6. [Working with Dependencies](#working-with-dependencies)
7. [Running and Proving Your Program](#running-and-proving-your-program)
8. [Advanced Topics](#advanced-topics)
9. [Common Patterns and Best Practices](#common-patterns-and-best-practices)
10. [Troubleshooting](#troubleshooting)

## Understanding the Architecture

ProofLab simplifies writing zero-knowledge proofs by providing a framework that abstracts away the complexities of zkVM integration. The project is organized as a Cargo workspace with multiple crates:

- **prooflab**: Main CLI tool and zkVM integration
- **prooflab_io**: I/O marshalling between host and guest programs

At its core, ProofLab separates code into:

- **Guest code**: Runs inside the zkVM and generates proofs (your `main()` function)
- **Host code**: Runs outside the zkVM to provide inputs and process outputs (your `input()` and `output()` functions)

This isolation ensures clear boundaries between the proven code and the supporting infrastructure.

![ProofLab Execution Flow](../assets/prooflab_execution_flow.png)

## Project Structure

A basic ProofLab project follows this structure:

```
my_zkvm_program/
├── Cargo.toml         # Dependencies for your program
└── src/
    └── main.rs        # Contains main(), input(), and output() functions
```

You can also create a more complex project with a library:

```
my_zkvm_program/
├── Cargo.toml         # Dependencies for your program
├── lib/
│   ├── Cargo.toml     # Dependencies for your library
│   └── src/
│       └── lib.rs     # Shared code to be used by your main program
└── src/
    └── main.rs        # Contains main(), input(), and output() functions
```

## Creating a New Project

Start by creating a new Rust project:

```bash
# Create a new project directory
cargo new my_zkvm_program
cd my_zkvm_program
```

Update your `Cargo.toml` to include the `prooflab_io` dependency:

```toml
[package]
name = "my_zkvm_program"
version = "0.1.0"
edition = "2021"

[dependencies]
# Other dependencies your code needs
prooflab_io = { git = "https://github.com/ProofLabDev/prooflab-rs.git", package = "prooflab_io" }
```

## The Three Core Functions

Every ProofLab project must implement a `main()` function and can optionally implement `input()` and `output()` functions.

### The `main()` Function

This function runs inside the zkVM and contains the core logic you want to prove:

```rust
use prooflab_io;

fn main() {
    // Read inputs from the host
    let input_value: u32 = prooflab_io::read();
    
    // Your core computation logic
    let result = input_value * 2;
    
    // Commit results to be read by the host
    prooflab_io::commit(&result);
}
```

### The `input()` Function (Optional)

This function runs on the host before the zkVM execution and prepares inputs:

```rust
use prooflab_io;

fn input() {
    // Define input data for the zkVM
    let value = 42u32;
    
    // Send it to the zkVM
    prooflab_io::write(&value);
}
```

### The `output()` Function (Optional)

This function runs on the host after the zkVM execution and processes the results:

```rust
use prooflab_io;

fn output() {
    // Read the committed data from the zkVM
    let result: u32 = prooflab_io::out();
    
    // Process the output
    println!("The result is: {}", result);
}
```

## Data Flow Between Host and Guest

ProofLab uses a simple I/O API through the `prooflab_io` crate:

### Sending Data to the zkVM (Host → Guest)

In your `input()` function, use `prooflab_io::write()` to send data:

```rust
fn input() {
    // You can write multiple values
    let a = 5u32;
    let b = 10u32;
    let c = "hello".to_string();
    
    prooflab_io::write(&a);
    prooflab_io::write(&b);
    prooflab_io::write(&c);
}
```

### Reading Data in the zkVM (Guest)

In your `main()` function, use `prooflab_io::read()` to get the input data:

```rust
fn main() {
    // Read values in the same order they were written
    let a: u32 = prooflab_io::read();
    let b: u32 = prooflab_io::read();
    let c: String = prooflab_io::read();
    
    // Process the data...
}
```

### Sending Data from the zkVM (Guest → Host)

In your `main()` function, use `prooflab_io::commit()` to commit results:

```rust
fn main() {
    // ... computation ...
    
    // You can commit multiple values
    let result1 = 42u32;
    let result2 = "done".to_string();
    
    prooflab_io::commit(&result1);
    prooflab_io::commit(&result2);
}
```

### Reading zkVM Results (Host)

In your `output()` function, use `prooflab_io::out()` to get the committed data:

```rust
fn output() {
    // Option 1: Read each value separately
    let result1: u32 = prooflab_io::out();
    let result2: String = prooflab_io::out();
    
    println!("Result 1: {}", result1);
    println!("Result 2: {}", result2);
    
    // Option 2: Read multiple values as a tuple
    // let (result1, result2): (u32, String) = prooflab_io::out();
}
```

## Example Programs

Here are some examples to help you understand the pattern:

### Example 1: Fibonacci Sequence

```rust
use prooflab_io;

fn main() {
    let n: u32 = prooflab_io::read();
    prooflab_io::commit(&n);

    let mut a: u32 = 0;
    let mut b: u32 = 1;
    for _ in 0..n {
        let mut c = a + b;
        c %= 7919; // Modulus to prevent overflow
        a = b;
        b = c;
    }

    prooflab_io::commit(&a);
    prooflab_io::commit(&b);
}

fn input() {
    let n = 1000u32;
    prooflab_io::write(&n);
}

fn output() {
    let (n, a, b): (u32, u32, u32) = prooflab_io::out();
    println!("n: {}", n);
    println!("a: {}", a);
    println!("b: {}", b);
}
```

### Example 2: Cryptographic Hash

```rust
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
```

### Example 3: Bubble Sort

```rust
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
```

## Working with Dependencies

You can use many standard Rust crates in your zkVM program. However, there are some limitations:

1. Standard I/O operations like `println!()` won't work inside the zkVM (in your `main()` function)
2. Some system calls may not be available inside the zkVM
3. Some crates may need special versions optimized for zkVMs

For best results, prefer pure computation libraries that don't rely on external resources.

### Using Accelerated Libraries

ProofLab-rs supports hardware-accelerated versions of common cryptographic libraries. To use them, add the `--precompiles` flag when running your program.

**SP1 Accelerated Crates:**
- sha2 v0.10.6
- sha3 v0.10.8
- crypto-bigint v0.5.5
- tiny-keccak v2.0.2
- ed25519-consensus v2.1.0
- ecdsa-core v0.16.9

**RISC0 Accelerated Crates:**
- sha2 v0.10.6
- k256 v0.13.1
- crypto-bigint v0.5.5

## Running and Proving Your Program

Once your code is ready, you can generate a zero-knowledge proof of its execution:

### Using SP1 Backend

```bash
cargo run --release -p prooflab -- prove-sp1 /path/to/my_zkvm_program
```

### Using RISC0 Backend

```bash
cargo run --release -p prooflab -- prove-risc0 /path/to/my_zkvm_program
```

### Using GPU Acceleration

For faster proof generation, you can use GPU acceleration (requires compatible hardware):

```bash
cargo run --release -p prooflab -- prove-sp1 /path/to/my_zkvm_program --gpu
```

### Submitting Proofs to Aligned

If you want to submit your proofs to the Aligned Layer verification system:

```bash
cargo run --release -p prooflab -- prove-sp1 /path/to/my_zkvm_program --submit-to-aligned --keystore-path /path/to/keystore.json
```

## Advanced Topics

### Handling Complex Data Structures

For complex data types, ensure they implement Serde's `Serialize` and `Deserialize` traits:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct MyData {
    id: u32,
    name: String,
    values: Vec<f64>,
}

fn main() {
    let data: MyData = prooflab_io::read();
    // Process the data...
    prooflab_io::commit(&data);
}

fn input() {
    let data = MyData {
        id: 1,
        name: "Example".to_string(),
        values: vec![1.0, 2.0, 3.0],
    };
    prooflab_io::write(&data);
}

fn output() {
    let data: MyData = prooflab_io::out();
    println!("Data: {:?}", data);
}
```

### Memory Management

zkVMs have limits on memory usage. For large computations:

1. Break processing into smaller chunks
2. Avoid excessive memory allocation
3. Use fixed-size arrays where possible
4. Consider memory-efficient algorithms

### Debugging zkVM Programs

Since zkVMs don't support traditional debugging, use these strategies:

1. Test outside the zkVM first by creating a normal Rust binary
2. Use the commit mechanism to output intermediate values
3. Break complex logic into smaller, testable functions
4. Add assertions to verify program correctness

## Common Patterns and Best Practices

### Keep Input and Output Functions Small

The `input()` and `output()` functions should primarily handle data preparation and presentation, not complex logic:

```rust
fn input() {
    // Good: Simple data preparation
    let data = load_data_from_file("input.json");
    prooflab_io::write(&data);
}

fn output() {
    // Good: Simple result presentation
    let result = prooflab_io::out();
    save_results_to_file("output.json", &result);
}
```

## Library Support

For larger projects, moving shared code to a library helps with organization and reusability:

```
my_project/
├── Cargo.toml          # Main project manifest
├── lib/
│   ├── Cargo.toml      # Library manifest
│   └── src/
│       └── lib.rs      # Shared utility functions and data structures
└── src/
    └── main.rs         # Contains main(), input(), and output() functions
```

### Creating and Using a Library

#### 1. Set up the library crate

First, create a library directory and initialize it:

```bash
mkdir -p lib/src
```

Create `lib/Cargo.toml`:

```toml
[package]
name = "my_project_lib"
version = "0.1.0"
edition = "2021"

[dependencies]
# Add library-specific dependencies
serde = { version = "1.0", features = ["derive"] }
```

#### 2. Implement your library

Create `lib/src/lib.rs` with your shared functionality:

```rust
use serde::{Serialize, Deserialize};

// Define data structures
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: u64,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub timestamp: u64,
}

// Implement utility functions
pub fn verify_transaction(tx: &Transaction) -> bool {
    // Example validation logic
    tx.amount > 0 && !tx.sender.is_empty() && !tx.receiver.is_empty()
}

pub fn calculate_fee(tx: &Transaction) -> u64 {
    tx.amount / 100  // Example: 1% fee
}

// More complex functions
pub fn batch_verify_transactions(txs: &[Transaction]) -> Vec<bool> {
    txs.iter().map(|tx| verify_transaction(tx)).collect()
}
```

#### 3. Configure the main crate to use the library

Update your main `Cargo.toml`:

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"

[dependencies]
prooflab_io = { git = "https://github.com/ProofLabDev/prooflab-rs.git", package = "prooflab_io" }
my_project_lib = { path = "./lib" }
serde = { version = "1.0", features = ["derive"] }
```

#### 4. Use the library in your main program

Now you can use the library in your `src/main.rs`:

```rust
use prooflab_io;
use my_project_lib::{Transaction, verify_transaction, batch_verify_transactions};
use serde::{Serialize, Deserialize};

fn main() {
    // Read transactions from the host
    let transactions: Vec<Transaction> = prooflab_io::read();
    
    // Use library functions to process them
    let verification_results = batch_verify_transactions(&transactions);
    
    // Calculate total fees
    let total_fee: u64 = transactions.iter()
        .filter(|tx| verify_transaction(tx))
        .map(|tx| my_project_lib::calculate_fee(tx))
        .sum();
    
    // Commit results
    prooflab_io::commit(&verification_results);
    prooflab_io::commit(&total_fee);
}

fn input() {
    // Create sample transactions
    let transactions = vec![
        Transaction {
            id: 1,
            sender: "Alice".to_string(),
            receiver: "Bob".to_string(),
            amount: 1000,
            timestamp: 1677592291,
        },
        Transaction {
            id: 2,
            sender: "Charlie".to_string(),
            receiver: "Dave".to_string(),
            amount: 500,
            timestamp: 1677592350,
        },
        Transaction {
            id: 3,
            sender: "".to_string(),  // Invalid transaction
            receiver: "Eve".to_string(),
            amount: 200,
            timestamp: 1677592400,
        },
    ];
    
    prooflab_io::write(&transactions);
}

fn output() {
    let (verification_results, total_fee): (Vec<bool>, u64) = prooflab_io::out();
    
    println!("Transaction verification results:");
    for (i, is_valid) in verification_results.iter().enumerate() {
        println!("Transaction {}: {}", i + 1, if *is_valid { "Valid" } else { "Invalid" });
    }
    
    println!("Total fees collected: {}", total_fee);
}
```

### When to Use Libraries

Libraries are especially useful for:

1. **Shared Data Structures**: Define transaction formats, blockchain states, etc.
2. **Common Algorithms**: Cryptographic primitives, validation routines, etc.
3. **Business Logic**: Rules and processes that need to be consistently applied
4. **Testing**: Libraries can be tested independently of the zkVM environment

### Testing Your Library

Since library code is pure Rust, you can test it using standard Rust testing tools:

```rust
// In lib/src/lib.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verify_transaction() {
        let valid_tx = Transaction {
            id: 1,
            sender: "Alice".to_string(),
            receiver: "Bob".to_string(),
            amount: 1000,
            timestamp: 1677592291,
        };
        
        let invalid_tx = Transaction {
            id: 2,
            sender: "".to_string(),
            receiver: "Dave".to_string(),
            amount: 500,
            timestamp: 1677592350,
        };
        
        assert!(verify_transaction(&valid_tx));
        assert!(!verify_transaction(&invalid_tx));
    }
    
    #[test]
    fn test_calculate_fee() {
        let tx = Transaction {
            id: 1,
            sender: "Alice".to_string(),
            receiver: "Bob".to_string(),
            amount: 1000,
            timestamp: 1677592291,
        };
        
        assert_eq!(calculate_fee(&tx), 10); // 1% of 1000
    }
}
```

Test your library with:

```bash
cd lib
cargo test
```

### Use Clear Type Annotations

Always use explicit type annotations to avoid confusion:

```rust
// Good: Type is explicitly stated
let input_value: u32 = prooflab_io::read();

// Avoid: Implicit typing
let input_value = prooflab_io::read();  // What type is this?
```

### Handle Errors Gracefully

Since panics inside the zkVM will fail proof generation, handle errors gracefully:

```rust
fn main() {
    let input: Result<Vec<u8>, String> = prooflab_io::read();
    
    match input {
        Ok(data) => {
            // Process valid data
            let result = process_data(data);
            prooflab_io::commit(&Ok::<_, String>(result));
        },
        Err(err) => {
            // Handle error case
            prooflab_io::commit(&Err::<Vec<u8>, _>(err));
        }
    }
}
```

## Troubleshooting

### Common Issues and Solutions

1. **Proof generation fails with memory error**
   - Reduce memory usage in your program
   - Break large computations into smaller parts

2. **Cryptographic operations are slow**
   - Use the `--precompiles` flag for hardware acceleration
   - Ensure you're using compatible versions of crypto libraries

3. **Type mismatch errors**
   - Ensure types match exactly between `write()` and `read()` calls
   - Use explicit type annotations

4. **Proof verification fails**
   - Ensure deterministic computation (avoid random numbers)
   - Check for undefined behavior or race conditions

5. **Build errors with dependencies**
   - Some crates may not be compatible with zkVMs
   - Check for zkVM-specific versions of libraries

### Getting Help

If you encounter issues not covered here:

1. Check the [ProofLab GitHub repository](https://github.com/ProofLabDev/prooflab-rs) for updates
2. Join the [Telegram support group](https://t.me/+7Qd3EutBDwZhM2U5)
3. Review the examples provided with ProofLab for reference implementations
4. Look at the test suite in the `crates/prooflab/tests` directory to understand the internals

## Conclusion

ProofLab provides a simplified way to develop zero-knowledge applications while abstracting away the complexities of zkVM integration. By following the patterns in this guide, you can create efficient and effective zkVM programs with your choice of backend.

Remember the key workflow:
1. Define your computation in `main()`
2. Prepare inputs in `input()`
3. Process outputs in `output()`
4. Generate proofs with your preferred zkVM

Happy proving with ProofLab!