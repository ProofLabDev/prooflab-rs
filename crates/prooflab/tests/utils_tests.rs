use prooflab::utils;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tempfile::{tempdir, NamedTempFile};

// Test the prepend function
#[test]
fn test_prepend() {
    // Create a temporary file with some content
    let mut temp_file = NamedTempFile::new().unwrap();
    let original_content = "Original content";
    temp_file.write_all(original_content.as_bytes()).unwrap();
    
    // Get the path as a string
    let temp_path = temp_file.path().to_str().unwrap().to_string();
    
    // Prepend some text
    let prepend_text = "Prepended text\n";
    utils::prepend(&temp_path, prepend_text).unwrap();
    
    // Read the file content after prepending
    let mut file = File::open(&temp_path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    
    // Verify the content
    let expected_content = format!("{}{}", prepend_text, original_content);
    assert_eq!(content, expected_content);
}

// Test the replace function
#[test]
fn test_replace() {
    // Create a temporary file with some content
    let mut temp_file = NamedTempFile::new().unwrap();
    let original_content = "This is a test content with replaceable text";
    temp_file.write_all(original_content.as_bytes()).unwrap();
    
    // Get the path
    let temp_path = PathBuf::from(temp_file.path());
    
    // Replace text
    let search = "replaceable";
    let replacement = "replaced";
    utils::replace(&temp_path, search, replacement).unwrap();
    
    // Read the file content after replacement
    let mut file = File::open(&temp_path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    
    // Verify the content
    let expected_content = "This is a test content with replaced text";
    assert_eq!(content, expected_content);
}

// Test validate_directory_structure function when available
// Note: This might need to be mocked with a specific file structure
#[test]
fn test_validate_directory_structure() {
    // Create a temporary directory with the expected structure
    let temp_dir = tempdir().unwrap();
    let dir_path = temp_dir.path();
    
    // Create the src directory
    let src_dir = dir_path.join("src");
    fs::create_dir(&src_dir).unwrap();
    
    // Create a main.rs file in src
    let main_file_path = src_dir.join("main.rs");
    let mut main_file = File::create(&main_file_path).unwrap();
    main_file.write_all(b"pub fn main() { println!(\"Hello\"); }").unwrap();
    
    // Create Cargo.toml file
    let cargo_file_path = dir_path.join("Cargo.toml");
    let mut cargo_file = File::create(&cargo_file_path).unwrap();
    cargo_file.write_all(b"[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
    
    // Test the validation function
    let result = utils::validate_directory_structure(&dir_path.to_string_lossy());
    assert!(result, "Directory structure validation failed");
}

// Test extract_function_bodies when available
#[test]
fn test_extract_function_bodies() {
    // Create a temporary file with function definitions
    let mut temp_file = NamedTempFile::new().unwrap();
    let content = r#"
    use std::println;

    pub fn main() {
        println!("Main function");
    }

    pub fn input() {
        println!("Input function");
    }

    pub fn output() {
        println!("Output function");
    }
    "#;
    temp_file.write_all(content.as_bytes()).unwrap();
    
    // Get the path
    let temp_path = PathBuf::from(temp_file.path());
    
    // Extract function bodies
    let function_names = vec![
        "fn main()".to_string(),
        "fn input()".to_string(),
        "fn output()".to_string(),
    ];
    
    let result = utils::extract_function_bodies(&temp_path, function_names).unwrap();
    
    // Verify results
    assert_eq!(result.len(), 3, "Should extract 3 function bodies");
    assert!(result[0].contains("println!(\"Main function\")"), "Main function body not correctly extracted");
    assert!(result[1].contains("println!(\"Input function\")"), "Input function body not correctly extracted");
    assert!(result[2].contains("println!(\"Output function\")"), "Output function body not correctly extracted");
}

// Test get_imports function when available
#[test]
fn test_get_imports() {
    // Create a temporary file with imports
    let mut temp_file = NamedTempFile::new().unwrap();
    let content = r#"
    use std::io;
    use regex::Regex;
    use prooflab_io;
    
    pub fn main() {
        println!("Hello");
    }
    "#;
    temp_file.write_all(content.as_bytes()).unwrap();
    
    // Get the path
    let temp_path = PathBuf::from(temp_file.path());
    
    // Get imports - function returns a String with all imports
    let imports = utils::get_imports(&temp_path).unwrap();
    
    // Verify results by checking the content of the string
    assert!(imports.contains("use std::io;"), "Missing std::io import");
    assert!(imports.contains("use regex::Regex;"), "Missing regex::Regex import");
    assert!(imports.contains("use prooflab_io;"), "Missing prooflab_io import");
    
    // Count the number of import lines
    let import_count = imports.lines().filter(|line| line.trim().starts_with("use ")).count();
    assert!(import_count >= 3, "Should find at least 3 imports");
}