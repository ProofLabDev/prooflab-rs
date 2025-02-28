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

fn copy_patch_section(toml_path: &Path, guest_toml_path: &Path) -> io::Result<()> {
    // Read source toml
    let mut source_toml = std::fs::File::open(toml_path)?;
    let mut source_content = String::new();
    source_toml.read_to_string(&mut source_content)?;

    // Read destination toml
    let mut dest_toml = std::fs::File::open(guest_toml_path)?;
    let mut dest_content = String::new();
    dest_toml.read_to_string(&mut dest_content)?;

    // Look for [patch.crates-io] section
    match source_content.find("[patch.crates-io]") {
        Some(start_index) => {
            // Get patch section from source
            let source_patches = &source_content[start_index..];

            // Find the end of patch section (next section or end of file)
            let end_index = source_patches.find("\n[").unwrap_or(source_patches.len());
            let source_patches = &source_patches[..end_index];

            // Parse patches into complete section
            let mut patch_section = String::from(source_patches.trim());
            if !patch_section.ends_with('\n') {
                patch_section.push('\n');
            }

            // Check if destination already has a patch section
            if !dest_content.contains("[patch.crates-io]") {
                // Append the patch section with a newline before
                let mut dest_file = OpenOptions::new().append(true).open(guest_toml_path)?;
                writeln!(dest_file, "\n{}", patch_section)?;
            } else {
                // We need to merge the patch sections
                // Read the existing patch section
                let dest_start = dest_content.find("[patch.crates-io]").unwrap();
                let dest_patches = &dest_content[dest_start..];
                let dest_end = dest_patches.find("\n[").unwrap_or(dest_patches.len());
                let dest_patches = &dest_patches[..dest_end];

                // Parse source patches into individual crate entries
                let patch_lines: Vec<&str> = source_patches
                    .lines()
                    .skip(1) // Skip the [patch.crates-io] line
                    .map(|s| s.trim())
                    .filter(|line| !line.is_empty())
                    .collect();

                // Extract crate names from source patches
                let mut new_patches = String::new();
                for line in patch_lines {
                    // Extract crate name from line like "crate_name = { ... }"
                    if let Some(crate_name) = line.split('=').next() {
                        let crate_name = crate_name.trim();
                        // Check if this crate is already patched in destination
                        if !dest_patches.contains(&format!("{} =", crate_name)) {
                            if !new_patches.is_empty() {
                                new_patches.push('\n');
                            }
                            new_patches.push_str(line);
                        }
                    }
                }

                if !new_patches.is_empty() {
                    // We need to append the new patches to the existing patch section
                    // First, read the whole file
                    let mut updated_content = dest_content.clone();
                    
                    // Insert new patches at the end of the existing patch section
                    let insert_pos = dest_start + dest_end;
                    updated_content.insert_str(insert_pos, &format!("\n{}", new_patches));
                    
                    // Write back the updated content
                    let mut dest_file = fs::File::create(guest_toml_path)?;
                    dest_file.write_all(updated_content.as_bytes())?;
                }
            }
            
            Ok(())
        }
        None => Ok(()),  // No patch section to copy, that's ok
    }
}

/// Copies auxiliary files referenced by include_bytes! and similar macros
fn copy_auxiliary_files(guest_path: &Path, workspace_guest_dir: &Path, workspace_host_dir: &Path) -> io::Result<()> {
    // Read all Rust source files to find include_bytes! and include_str! macros
    let src_dir = guest_path.join("src");
    let mut include_patterns = Vec::new();
    
    if src_dir.exists() {
        visit_dir_for_includes(&src_dir, &mut include_patterns)?;
    }
    
    // Process each include pattern found
    for path_str in include_patterns {
        // Remove quotes and whitespace
        let path_str = path_str.trim().trim_matches('"').trim_matches('\'');
        
        // Make relative paths absolute from the guest path
        let abs_source_path = if Path::new(path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            // For patterns like "../file.bin", resolve relative to src directory
            if path_str.starts_with("../") {
                guest_path.join(path_str.trim_start_matches("../"))
            } else {
                guest_path.join("src").join(path_str)
            }
        };
        
        // Skip if file doesn't exist
        if !abs_source_path.exists() {
            error!("Referenced file not found: {:?}", abs_source_path);
            continue;
        }
        
        // Determine the destination path in workspace
        let rel_path = if let Ok(rel) = abs_source_path.strip_prefix(guest_path) {
            rel
        } else {
            // For files outside the guest path, place them at the root of workspace
            Path::new(abs_source_path.file_name().unwrap_or_default())
        };
        
        let workspace_guest_dest = workspace_guest_dir.join(rel_path);
        let workspace_host_dest = workspace_host_dir.join(rel_path);
        
        // Create parent directories if needed
        if let Some(parent) = workspace_guest_dest.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = workspace_host_dest.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Copy the file
        fs::copy(&abs_source_path, &workspace_guest_dest)?;
        fs::copy(&abs_source_path, &workspace_host_dest)?;
        println!("Copied auxiliary file: {:?} -> {:?}", abs_source_path, workspace_guest_dest);
    }
    
    Ok(())
}

/// Recursively searches Rust files for include_bytes! and include_str! directives
fn visit_dir_for_includes(dir: &Path, patterns: &mut Vec<String>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                visit_dir_for_includes(&path, patterns)?;
            } else if let Some(ext) = path.extension() {
                if ext == "rs" {
                    find_includes_in_file(&path, patterns)?;
                }
            }
        }
    }
    Ok(())
}

/// Searches a Rust file for include_bytes! and include_str! directives
fn find_includes_in_file(file_path: &Path, patterns: &mut Vec<String>) -> io::Result<()> {
    let content = fs::read_to_string(file_path)?;
    
    // Regular expressions to find include_bytes! and include_str! macros
    let re_bytes = Regex::new(r#"include_bytes!\s*\(\s*(["'].*?["'])\s*\)"#).unwrap();
    let re_str = Regex::new(r#"include_str!\s*\(\s*(["'].*?["'])\s*\)"#).unwrap();
    
    // Extract the paths from include_bytes! macros
    for cap in re_bytes.captures_iter(&content) {
        if let Some(m) = cap.get(1) {
            patterns.push(m.as_str().to_string());
        }
    }
    
    // Extract the paths from include_str! macros
    for cap in re_str.captures_iter(&content) {
        if let Some(m) = cap.get(1) {
            patterns.push(m.as_str().to_string());
        }
    }
    
    Ok(())
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
    // Check if base files exist
    if !Path::new(base_host_toml_dir).exists() || !Path::new(base_guest_toml_dir).exists() {
        error!("Required base template files not found. Please install ProofLab using the installation script.");
        error!("Missing files:");
        if !Path::new(base_host_toml_dir).exists() {
            error!("  - {:?}", base_host_toml_dir);
        }
        if !Path::new(base_guest_toml_dir).exists() {
            error!("  - {:?}", base_guest_toml_dir);
        }
        error!("Try running the install script again: 'curl -L https://raw.githubusercontent.com/ProofLabDev/prooflab-rs/main/install_prooflab.sh | bash'");
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Required base template files not found. Please reinstall ProofLab.",
        ));
    }
    
    let workspace_guest_src_dir = workspace_guest_dir.join("src");
    let workspace_host_src_dir = workspace_host_dir.join("src");

    // Create directories if they don't exist
    fs::create_dir_all(&workspace_guest_src_dir)?;
    fs::create_dir_all(&workspace_host_src_dir)?;

    // Clean up old files except metrics.rs
    if let Ok(entries) = fs::read_dir(&workspace_guest_src_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.file_name().unwrap_or_default() != "metrics.rs" {
                    if path.is_file() {
                        let _ = fs::remove_file(&path);
                    } else if path.is_dir() {
                        let _ = fs::remove_dir_all(&path);
                    }
                }
            }
        }
    }
    
    if let Ok(entries) = fs::read_dir(&workspace_host_src_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.file_name().unwrap_or_default() != "metrics.rs" {
                    if path.is_file() {
                        let _ = fs::remove_file(&path);
                    } else if path.is_dir() {
                        let _ = fs::remove_dir_all(&path);
                    }
                }
            }
        }
    }

    // Copy src/ directory contents, skipping metrics.rs if it exists in destination
    let src_dir_path = guest_path.join("src");
    if let Ok(entries) = fs::read_dir(&src_dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let file_name = path.file_name().unwrap_or_default();
                if file_name != "metrics.rs" {
                    let guest_dest = workspace_guest_src_dir.join(file_name);
                    let host_dest = workspace_host_src_dir.join(file_name);
                    
                    if path.is_file() {
                        let _ = fs::copy(&path, &guest_dest);
                        let _ = fs::copy(&path, &host_dest);
                    } else if path.is_dir() {
                        let _ = copy_dir_all(&path, &guest_dest);
                        let _ = copy_dir_all(&path, &host_dest);
                    }
                }
            }
        }
    }

    // Copy lib/ if present
    let lib_dir_path = guest_path.join("lib");
    if Path::new(&lib_dir_path).exists() {
        let workspace_guest_lib_dir = workspace_guest_dir.join("lib");
        let workspace_host_lib_dir = workspace_host_dir.join("lib");
        let _ = copy_dir_all(&lib_dir_path, workspace_guest_lib_dir);
        let _ = copy_dir_all(&lib_dir_path, workspace_host_lib_dir);
    }

    // Copy auxiliary files referenced in include_bytes! and include_str! macros
    if let Err(e) = copy_auxiliary_files(guest_path, workspace_guest_dir, workspace_host_dir) {
        error!("Failed to copy auxiliary files: {:?}", e);
        // Non-fatal error, continue with workspace setup
    }

    // Copy Cargo.toml for zkVM
    match fs::copy(base_guest_toml_dir, program_toml_dir) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to copy guest toml from {:?} to {:?}: {:?}", base_guest_toml_dir, program_toml_dir, e);
            return Err(e);
        }
    }
    
    match fs::copy(base_host_toml_dir, host_toml_dir) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to copy host toml from {:?} to {:?}: {:?}", base_host_toml_dir, host_toml_dir, e);
            return Err(e);
        }
    }

    // Select dependencies and patches from the original Cargo.toml
    let toml_path = guest_path.join("Cargo.toml");
    if let Err(e) = copy_dependencies(&toml_path, program_toml_dir) {
        error!("Failed to copy dependencies to program toml: {:?}", e);
        return Err(e);
    }
    
    if let Err(e) = copy_dependencies(&toml_path, host_toml_dir) {
        error!("Failed to copy dependencies to host toml: {:?}", e);
        return Err(e);
    }

    // Copy patch sections
    if let Err(e) = copy_patch_section(&toml_path, program_toml_dir) {
        error!("Failed to copy patch section to program toml: {:?}", e);
        return Err(e);
    }
    
    if let Err(e) = copy_patch_section(&toml_path, host_toml_dir) {
        error!("Failed to copy patch section to host toml: {:?}", e);
        return Err(e);
    }
    
    // Handle lib directory if it exists - copy patches from lib Cargo.toml as well
    let lib_dir_path = guest_path.join("lib");
    let lib_toml_path = lib_dir_path.join("Cargo.toml");
    if Path::new(&lib_toml_path).exists() {
        // Copy patches from lib/Cargo.toml
        if let Err(e) = copy_patch_section(&lib_toml_path, program_toml_dir) {
            error!("Failed to copy patch section from lib toml to program toml: {:?}", e);
            return Err(e);
        }
        
        if let Err(e) = copy_patch_section(&lib_toml_path, host_toml_dir) {
            error!("Failed to copy patch section from lib toml to host toml: {:?}", e);
            return Err(e);
        }
    }

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
    
    // Log the actual path being checked
    println!("Checking directory structure at path: {:?}", root);
    
    // Check if Cargo.toml exists in the root directory
    let cargo_toml = root.join("Cargo.toml");
    println!("Looking for Cargo.toml at: {:?}", cargo_toml);
    if !cargo_toml.exists() {
        error!("Cargo.toml not found at {:?}", cargo_toml);
        return false;
    }

    // Check if src/ directory exists
    let src_dir = root.join("src");
    println!("Looking for src directory at: {:?}", src_dir);
    if !src_dir.exists() {
        error!("src/ directory not found at {:?}", src_dir);
        return false;
    }

    // Check if src/ contains main.rs file
    let main_rs = src_dir.join("main.rs");
    println!("Looking for main.rs at: {:?}", main_rs);
    if !main_rs.exists() {
        error!("main.rs not found at {:?}", main_rs);
        return false;
    }

    println!("Directory structure validation successful!");
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
