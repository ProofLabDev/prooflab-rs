use regex::Regex;
use prooflab_io;

pub fn main() {
    // Read two inputs from the prover: a regex pattern and a target string.
    let pattern: String = prooflab_io::read();
    let target_string: String = prooflab_io::read();

    // Try to compile the regex pattern. If it fails, write `false` as output and return.
    let regex = match Regex::new(&pattern) {
        Ok(regex) => regex,
        Err(_) => {
            panic!("Invalid regex pattern");
        }
    };

    // Perform the regex search on the target string.
    let result = regex.is_match(&target_string);

    // Write the result (true or false) to the output.
    prooflab_io::commit(&result);
}

pub fn input() {
    let pattern = "a+".to_string();
    let target_string = "an era of truth, not trust".to_string();

    // Write in a simple regex pattern.
    prooflab_io::write(&pattern);
    prooflab_io::write(&target_string);
}

pub fn output() {
    // Read the output.
    let res: bool = prooflab_io::out();
    println!("res: {}", res);
}
