use log::error;
use regex::Regex;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, ErrorKind, Read, Seek, Write},
    path::{Path, PathBuf},
};

// Host
pub const IO_WRITE: &str = "prooflab_io::write";
pub const IO_OUT: &str = "prooflab_io::out();";
pub const HOST_INPUT: &str = "// INPUT //";
pub const HOST_OUTPUT: &str = "// OUTPUT //";

// I/O Markers
pub const IO_READ: &str = "prooflab_io::read();";
pub const IO_COMMIT: &str = "prooflab_io::commit";

pub const OUTPUT_FUNC: &str = r"pub fn output() {";
pub const INPUT_FUNC: &str = r"pub fn input() {";

pub fn prepend(file_path: &str, text_to_prepend: &str) -> io::Result<()> {
    // Open the file in read mode to read its existing content
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    // Read the existing content of the file
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Move the file cursor to the beginning of the file
    file.seek(io::SeekFrom::Start(0))?;

    // Write the text to prepend followed by the existing content back to the file
    file.write_all(text_to_prepend.as_bytes())?;
    file.write_all(content.as_bytes())?;
    file.flush()?;

    Ok(())
}

pub fn replace(file_path: &PathBuf, search_string: &str, replace_string: &str) -> io::Result<()> {
    // Read the contents of the file
    let mut contents = String::new();
    fs::File::open(file_path)?.read_to_string(&mut contents)?;

    // Replace all occurrences of the search string with the replace string
    let new_contents = contents.replace(search_string, replace_string);

    // Write the new contents back to the file
    let mut file = fs::File::create(file_path)?;
    file.write_all(new_contents.as_bytes())?;

    Ok(())
}

fn copy_dir_all(src: &impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn insert(target_file: &str, text: &str, search_string: &str) -> io::Result<()> {
    // Read the contents of the target file
    let mut target_contents = String::new();
    fs::File::open(target_file)?.read_to_string(&mut target_contents)?;

    // Find the position of the search string in the target file
    if let Some(pos) = target_contents.find(search_string) {
        // Split the target contents into two parts
        let (before, after) = target_contents.split_at(pos + search_string.len());

        // Combine the parts with the insert contents
        let new_contents = format!("{}\n{}\n{}", before, text, after);

        // Write the new contents back to the target file
        let mut file = fs::File::create(target_file)?;
        file.write_all(new_contents.as_bytes())?;
    } else {
        println!("Search string not found in target file.");
    }

    Ok(())
}

//Note: Works with a one off '{' not with '}'
pub fn extract_function_bodies(
    file_path: &PathBuf,
    functions: Vec<String>,
) -> io::Result<Vec<String>> {
    // Read the contents of the target file
    let mut code = String::new();
    fs::File::open(file_path)?.read_to_string(&mut code)?;

    let mut start_indices = vec![];
    let mut index = 0;

    // Find all start indices of the function signature
    for keyword in functions {
        if let Some(start_index) = code[index..].find(&keyword) {
            let absolute_index = index + start_index;
            start_indices.push(absolute_index);
            index = absolute_index + keyword.len();
        }
    }

    // Extract the code for each function
    let mut extracted_codes = vec![];
    for &start_index in &start_indices {
        if let Some(start_brace_index) = code[start_index..].find('{') {
            let start_brace_index = start_index + start_brace_index;
            let mut stack = vec!["{"];
            let mut end_index = start_brace_index;

            for (i, ch) in code[start_brace_index + 1..].chars().enumerate() {
                if handle_stack(ch, &mut stack) {
                    end_index = start_brace_index + 1 + i;
                    break;
                }
            }

            let extracted_code = &code[start_brace_index + 1..end_index].trim();
            extracted_codes.push(extracted_code.to_string());
        }
    }

    Ok(extracted_codes)
}

// Function that handles the stack and status when parsing the file to extract_function_bodies
fn handle_stack(ch: char, stack: &mut Vec<&str>) -> bool {
    match stack.last() {
        Some(&"{") => return handle_char(ch, stack),
        Some(&"/") => match ch {
            '/' => {
                stack.pop();
                stack.push("//comment");
            }
            '*' => {
                stack.pop();
                stack.push("/*comment*\\");
            }
            _ => {
                stack.pop();
                handle_char(ch, stack);
            }
        },
        Some(&"//comment") => {
            if ch == '\n' {
                stack.pop();
            }
        }
        Some(&"/*comment*\\") => {
            if ch == '*' {
                stack.push("*");
            }
        }
        Some(&"*") => {
            match ch {
                '/' => {
                    stack.pop(); //pop("*")
                    stack.pop(); //pop("/*comment*\\")
                }
                _ => {
                    stack.pop(); //pop("*"), back to "/*comment*\\"
                }
            }
        }
        Some(&"\"string\"") => {
            if ch == '\"' {
                stack.pop();
            }
        }
        Some(&"\'c\'") => {
            if ch == '\'' {
                stack.pop();
            }
        }
        _ => {}
    }
    false
}
// Function to handle characters when in normal status of the stack
fn handle_char(ch: char, stack: &mut Vec<&str>) -> bool {
    match ch {
        '/' => {
            stack.push("/");
        }
        '{' => stack.push("{"),
        '}' => {
            stack.pop();
            if stack.is_empty() {
                return true;
            }
        }
        '\"' => {
            stack.push("\"string\"");
        }
        '\'' => {
            stack.push("\'c\'");
        }
        _ => {}
    }
    false
}

fn copy_dependencies(toml_path: &Path, guest_toml_path: &Path) -> io::Result<()> {
    // Read source toml
    let mut source_toml = std::fs::File::open(toml_path)?;
    let mut source_content = String::new();
    source_toml.read_to_string(&mut source_content)?;

    // Read destination toml
    let mut dest_toml = std::fs::File::open(guest_toml_path)?;
    let mut dest_content = String::new();
    dest_toml.read_to_string(&mut dest_content)?;

    match source_content.find("[dependencies]") {
        Some(start_index) => {
            // Get dependencies section from source
            let source_deps = &source_content[start_index + "[dependencies]".len()..];

            // Find the end of dependencies section (next section or end of file)
            let end_index = source_deps.find("\n[").unwrap_or(source_deps.len());
            let source_deps = &source_deps[..end_index];

            // Parse dependencies into individual entries
            let source_deps: Vec<&str> = source_deps
                .lines()
                .map(|s| s.trim())
                .filter(|line| !line.is_empty() && !line.starts_with('['))
                .collect();

            // Get existing dependencies from destination
            let existing_deps = if let Some(dest_start) = dest_content.find("[dependencies]") {
                let dest_deps = &dest_content[dest_start + "[dependencies]".len()..];
                let dest_end = dest_deps.find("\n[").unwrap_or(dest_deps.len());
                let dest_deps = &dest_deps[..dest_end];

                dest_deps
                    .lines()
                    .map(|s| s.trim())
                    .filter(|line| !line.is_empty() && !line.starts_with('['))
                    .collect::<Vec<&str>>()
            } else {
                Vec::new()
            };

            // Filter out duplicates and prepare new dependencies
            let new_deps: String = source_deps
                .into_iter()
                .filter(|dep| {
                    let dep_name = dep.split('=').next().unwrap_or("").trim();
                    !existing_deps
                        .iter()
                        .any(|existing| existing.split('=').next().unwrap_or("").trim() == dep_name)
                })
                .fold(String::new(), |mut acc, dep| {
                    if !acc.is_empty() {
                        acc.push('\n');
                    }
                    acc.push_str(dep);
                    acc
                });

            if !new_deps.is_empty() {
                // If destination doesn't have [dependencies] section, add it
                if !dest_content.contains("[dependencies]") {
                    let mut dest_file = OpenOptions::new().append(true).open(guest_toml_path)?;
                    writeln!(dest_file, "\n[dependencies]")?;
                }

                // Append new dependencies with proper newlines
                let mut dest_file = OpenOptions::new().append(true).open(guest_toml_path)?;

                // Add a newline before new dependencies if the file doesn't end with one
                if !dest_content.ends_with('\n') {
                    writeln!(dest_file)?;
                }

                writeln!(dest_file, "{}", new_deps)?;
                Ok(())
            } else {
                Ok(())
            }
        }
        None => Err(io::Error::other(
            "Failed to find `[dependencies]` in project Cargo.toml",
        )),
    }
}

pub fn prepare_workspace(
    guest_path: &Path,
    workspace_guest_dir: &Path,
    program_toml_dir: &Path,
    workspace_host_dir: &Path,
    host_toml_dir: &Path,
    base_host_toml_dir: &Path,
    base_guest_toml_dir: &Path,
) -> io::Result<()> {
    let workspace_guest_src_dir = workspace_guest_dir.join("src");
    let workspace_host_src_dir = workspace_host_dir.join("src");

    // Create directories if they don't exist
    fs::create_dir_all(&workspace_guest_src_dir)?;
    fs::create_dir_all(&workspace_host_src_dir)?;

    // Clean up old files except metrics.rs
    for entry in fs::read_dir(&workspace_guest_src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.file_name().unwrap_or_default() != "metrics.rs" {
            if path.is_file() {
                fs::remove_file(&path)?;
            } else if path.is_dir() {
                fs::remove_dir_all(&path)?;
            }
        }
    }
    for entry in fs::read_dir(&workspace_host_src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.file_name().unwrap_or_default() != "metrics.rs" {
            if path.is_file() {
                fs::remove_file(&path)?;
            } else if path.is_dir() {
                fs::remove_dir_all(&path)?;
            }
        }
    }

    // Copy src/ directory contents, skipping metrics.rs if it exists in destination
    let src_dir_path = guest_path.join("src");
    for entry in fs::read_dir(&src_dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default();
        if file_name != "metrics.rs" {
            let guest_dest = workspace_guest_src_dir.join(file_name);
            let host_dest = workspace_host_src_dir.join(file_name);
            if path.is_file() {
                fs::copy(&path, &guest_dest)?;
                fs::copy(&path, &host_dest)?;
            } else if path.is_dir() {
                copy_dir_all(&path, &guest_dest)?;
                copy_dir_all(&path, &host_dest)?;
            }
        }
    }

    // Copy lib/ if present
    let lib_dir_path = guest_path.join("lib");
    if Path::new(&lib_dir_path).exists() {
        let workspace_guest_lib_dir = workspace_guest_dir.join("lib");
        let workspace_host_lib_dir = workspace_host_dir.join("lib");
        copy_dir_all(&lib_dir_path, workspace_guest_lib_dir)?;
        copy_dir_all(&lib_dir_path, workspace_host_lib_dir)?;
    }

    // Copy Cargo.toml for zkVM
    fs::copy(base_guest_toml_dir, program_toml_dir)?;
    println!("{:?} {:?}", base_guest_toml_dir, program_toml_dir);
    fs::copy(base_host_toml_dir, host_toml_dir)?;

    // Select dependencies from the
    let toml_path = guest_path.join("Cargo.toml");
    copy_dependencies(&toml_path, program_toml_dir)?;
    copy_dependencies(&toml_path, host_toml_dir)?;

    Ok(())
}

//TODO: refactor this to eliminate the clone at each step.
pub fn get_imports(filename: &PathBuf) -> io::Result<String> {
    // Open the file
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    let mut imports = String::new();

    // Read the file line by line
    while let Some(line) = lines.next() {
        let mut line = line?;
        // Check if the line starts with "use "
        if line.trim_start().starts_with("use ")
            || line.trim_start().starts_with("pub mod ")
            || line.trim_start().starts_with("mod ")
        {
            line.push('\n');
            imports.push_str(&line.clone());
            // check if line does not contains a use declarator and a ';'
            // if not continue reading till one is found this covers the case where import statements cover multiple lines
            if !line.contains(';') {
                // Iterate and continue adding lines to the import while line does not contain a ';' break if it does
                for line in lines.by_ref() {
                    let mut line = line?;
                    line.push('\n');
                    imports.push_str(&line.clone());
                    if line.contains(';') {
                        break;
                    }
                }
            }
        }
    }

    Ok(imports)
}

pub fn extract_regex(file_path: &PathBuf, regex: &str) -> io::Result<Vec<String>> {
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut values = Vec::new();
    let regex = Regex::new(regex).map_err(io::Error::other)?;

    for line in reader.lines() {
        let line = line?;
        for cap in regex.captures_iter(&line) {
            if let Some(matched) = cap.get(1) {
                values.push(matched.as_str().to_string());
            }
        }
    }

    Ok(values)
}

//Change to remove regex and remove the marker
pub fn remove_lines(file_path: &PathBuf, target: &str) -> io::Result<()> {
    // Read the file line by line
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);

    // Collect lines that do not contain the target string
    let lines: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.contains(target))
        .collect();

    // Write the filtered lines back to the file
    let mut file = fs::File::create(file_path)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

pub fn validate_directory_structure(root: &str) -> bool {
    let root = Path::new(root);
    // Check if Cargo.toml exists in the root directory
    let cargo_toml = root.join("Cargo.toml");
    if !cargo_toml.exists() {
        error!("Cargo.toml not found.");
        return false;
    }

    // Check if src/ and lib/ directories exist
    let src_dir = root.join("src");

    if !src_dir.exists() {
        error!("src/ directory not found in root");
        return false;
    }

    // Check if src/ contains main.rs file
    let main_rs = src_dir.join("main.rs");
    if !main_rs.exists() {
        error!("main.rs not found in src/ directory in root");
        return false;
    }

    true
}

pub fn prepare_guest(
    imports: &str,
    main_func_code: &str,
    program_header: &str,
    io_read_header: &str,
    io_commit_header: &str,
    guest_main_file_path: &PathBuf,
) -> io::Result<()> {
    let mut guest_program = program_header.to_string();
    guest_program.push_str(imports);
    guest_program.push_str("pub fn main() {\n");
    guest_program
        .push_str("    println!(\"cycle-tracker-report-start: {}\", env!(\"CARGO_PKG_NAME\"));\n");
    guest_program.push_str(main_func_code);
    guest_program
        .push_str("\n    println!(\"cycle-tracker-report-end: {}\", env!(\"CARGO_PKG_NAME\"));\n");
    guest_program.push_str("}\n");

    // Replace prooflab::read()
    let guest_program = guest_program.replace(IO_READ, io_read_header);

    // Replace prooflab::commit()
    let guest_program = guest_program.replace(IO_COMMIT, io_commit_header);

    // Write to guest
    let mut file = fs::File::create(guest_main_file_path)?;
    file.write_all(guest_program.as_bytes())?;
    Ok(())
}
