use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Test case to scan the entire repository for sensitive client information
/// This test will fail if any sensitive data is detected in the codebase
#[cfg(test)]
mod sensitive_data_tests {
    use super::*;




    /// Test to scan entire repository for sensitive client information
    #[test]
    fn test_no_sensitive_client_data_in_repository() {
        let repo_root = find_repo_root().expect("Could not find repository root");
        let mut violations = Vec::new();

        // Scan all relevant files in the repository
        for entry in WalkDir::new(&repo_root)
            .into_iter()
            .filter_entry(|e| !is_excluded_dir(e.path()))
        {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            if should_scan_file(path) {
                if let Err(violation) = scan_file_for_sensitive_data_secure(path) {
                    violations.push(violation);
                }
            }
        }

        // Report any violations found
        if !violations.is_empty() {
            eprintln!("\n🚨 SENSITIVE DATA DETECTED IN REPOSITORY 🚨");
            eprintln!("The following files contain sensitive client information:");
            eprintln!();

            for violation in &violations {
                eprintln!("📁 File: {}", violation.file_path);
                eprintln!("   Line {}: {}", violation.line_number, violation.line_content);
                eprintln!("   Issue: {}", violation.issue_description);
                eprintln!();
            }

            eprintln!("❌ Repository contains sensitive client data!");
            eprintln!("Please remove or sanitize the above information before pushing to remote.");
            eprintln!();

            panic!("Sensitive client data detected in repository. See output above for details.");
        }

        println!("✅ No sensitive client data detected in repository");
    }

    /// Test to ensure no sensitive data in specific documentation files
    #[test]
    fn test_no_sensitive_data_in_documentation() {
        let repo_root = find_repo_root().expect("Could not find repository root");
        let doc_files = [
            "README.md",
            "ROADMAP.md",
            "RELEASE.md",
            "SECURITY.md",
            "SECURITY_ENHANCEMENTS.md",
            "HOMEBREW.md",
            "HOMEBREW_REPOSITORY_EXPLANATION.md",
            "HOMEBREW_TAP_TEST_SUCCESS.md",
            "HOMEBREW_TEST_REPORT.md",
            "WARP.md"
        ];

        let mut violations = Vec::new();

        for doc_file in &doc_files {
            let file_path = repo_root.join(doc_file);
            if file_path.exists() {
                if let Err(violation) = scan_file_for_sensitive_data_secure(&file_path) {
                    violations.push(violation);
                }
            }
        }

        if !violations.is_empty() {
            eprintln!("\n🚨 SENSITIVE DATA DETECTED IN DOCUMENTATION 🚨");
            for violation in &violations {
                eprintln!("📁 File: {}", violation.file_path);
                eprintln!("   Line {}: {}", violation.line_number, violation.line_content);
                eprintln!("   Issue: {}", violation.issue_description);
                eprintln!();
            }
            panic!("Sensitive client data detected in documentation files");
        }

        println!("✅ No sensitive client data detected in documentation files");
    }

    /// Test to ensure no sensitive data in source code
    #[test]
    fn test_no_sensitive_data_in_source_code() {
        let repo_root = find_repo_root().expect("Could not find repository root");
        let src_dir = repo_root.join("src");
        
        if !src_dir.exists() {
            return; // No source directory to scan
        }

        let mut violations = Vec::new();

        for entry in WalkDir::new(&src_dir) {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                if let Err(violation) = scan_file_for_sensitive_data_secure(path) {
                    violations.push(violation);
                }
            }
        }

        if !violations.is_empty() {
            eprintln!("\n🚨 SENSITIVE DATA DETECTED IN SOURCE CODE 🚨");
            for violation in &violations {
                eprintln!("📁 File: {}", violation.file_path);
                eprintln!("   Line {}: {}", violation.line_number, violation.line_content);
                eprintln!("   Issue: {}", violation.issue_description);
                eprintln!();
            }
            panic!("Sensitive client data detected in source code");
        }

        println!("✅ No sensitive client data detected in source code");
    }

    /// Test to ensure no sensitive data in configuration files
    #[test]
    fn test_no_sensitive_data_in_config_files() {
        let repo_root = find_repo_root().expect("Could not find repository root");
        let config_files = [
            "Cargo.toml",
            "Cargo.lock",
            ".pre-commit-config.yaml",
            ".github/workflows/ci.yml",
            ".github/workflows/pre-push.yml",
            ".github/workflows/security.yml"
        ];

        let mut violations = Vec::new();

        for config_file in &config_files {
            let file_path = repo_root.join(config_file);
            if file_path.exists() {
                if let Err(violation) = scan_file_for_sensitive_data_secure(&file_path) {
                    violations.push(violation);
                }
            }
        }

        if !violations.is_empty() {
            eprintln!("\n🚨 SENSITIVE DATA DETECTED IN CONFIG FILES 🚨");
            for violation in &violations {
                eprintln!("📁 File: {}", violation.file_path);
                eprintln!("   Line {}: {}", violation.line_number, violation.line_content);
                eprintln!("   Issue: {}", violation.issue_description);
                eprintln!();
            }
            panic!("Sensitive client data detected in configuration files");
        }

        println!("✅ No sensitive client data detected in configuration files");
    }
}

/// File extensions to scan for sensitive data
const SCANNABLE_EXTENSIONS: &[&str] = &[
    "md", "txt", "rs", "toml", "yaml", "yml", "json", "sh", "rb", "py", "js", "ts", "html", "css"
];

/// Directories to exclude from scanning
const EXCLUDE_DIRS: &[&str] = &[
    "target", ".git", "node_modules", ".cargo", "dist", "build", ".vscode", ".idea", "tests"
];

/// Files to exclude from scanning (sensitive data detection files themselves)
const EXCLUDE_FILES: &[&str] = &[
    "tests/sensitive_data_detection.rs",
    "scripts/sensitive-data-check.sh"
];

/// Check for patterns that indicate sensitive data without exposing the actual data
fn has_sensitive_indicators(content: &str) -> Vec<String> {
    let mut violations = Vec::new();
    
    // Check each line individually for more precise detection
    for line in content.lines() {
        // Check for specific user directory patterns (excluding generic ones)
        if line.contains("/Users/") && 
           !line.contains("/Users/user/") && 
           !line.contains("/Users/me/") &&
           !line.contains("/Users/hisgarden/") { // Allow hisgarden as it's the project owner
            violations.push("Contains specific user directory path (use generic /Users/user/ or /Users/me/ instead)".to_string());
            break; // Only report once per file
        }
        
        // Check for client directory patterns (excluding generic ones)
        if line.contains("Clients/") && 
           !line.contains("ClientA") && 
           !line.contains("ClientB") && 
           !line.contains("ClientC") &&
           !line.contains("/Users/user/workspace/Clients") && // Allow generic examples
           !line.contains("/Users/me/workspace/Clients") { // Allow generic examples
            violations.push("Contains specific client directory path (use generic ClientA, ClientB, ClientC instead)".to_string());
            break; // Only report once per file
        }
        
        // Check for workspace patterns that might be specific (but allow generic examples)
        if line.contains("workspace/Clients") && 
           !line.contains("/path/to/client/repositories") &&
           !line.contains("/Users/user/workspace/Clients") && // Allow generic examples
           !line.contains("/Users/me/workspace/Clients") { // Allow generic examples
            violations.push("Contains specific workspace path (use generic /path/to/client/repositories instead)".to_string());
            break; // Only report once per file
        }
    }
    
    violations
}

/// Represents a violation of sensitive data policy
#[derive(Debug)]
struct SensitiveDataViolation {
    file_path: String,
    line_number: usize,
    line_content: String,
    issue_description: String,
}

/// Find the repository root directory
fn find_repo_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut current_dir = std::env::current_dir()?;
    
    loop {
        if current_dir.join(".git").exists() {
            return Ok(current_dir);
        }
        
        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            return Err("Could not find repository root (no .git directory found)".into());
        }
    }
}

/// Check if a directory should be excluded from scanning
fn is_excluded_dir(path: &Path) -> bool {
    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
        EXCLUDE_DIRS.contains(&dir_name)
    } else {
        false
    }
}

/// Check if a file should be scanned for sensitive data
fn should_scan_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    // Check if this file should be excluded
    let path_str = path.to_string_lossy();
    for exclude_file in EXCLUDE_FILES {
        if path_str.contains(exclude_file) {
            return false;
        }
    }

    // Check file extension
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        SCANNABLE_EXTENSIONS.contains(&extension)
    } else {
        // Check for files without extensions that might be important
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            matches!(file_name, "Dockerfile" | "Makefile" | "LICENSE" | "CHANGELOG")
        } else {
            false
        }
    }
}

/// Scan a single file for sensitive data using secure patterns
fn scan_file_for_sensitive_data_secure(file_path: &Path) -> Result<(), SensitiveDataViolation> {
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => return Ok(()), // Skip files that can't be read as text
    };

    let file_path_str = file_path.to_string_lossy().to_string();

    // Check for sensitive indicators without exposing the actual sensitive data
    let violations = has_sensitive_indicators(&content);
    
    if !violations.is_empty() {
        // Find the first line that contains sensitive indicators
        for (line_number, line) in content.lines().enumerate() {
            let line_number = line_number + 1; // 1-based line numbers
            
            // Check if this line contains any of the violation patterns
            for violation in &violations {
                if line.contains("/Users/") && !line.contains("/Users/user/") && !line.contains("/Users/me/") {
                    return Err(SensitiveDataViolation {
                        file_path: file_path_str.clone(),
                        line_number,
                        line_content: line.to_string(),
                        issue_description: violation.clone(),
                    });
                }
                
                if line.contains("Clients/") && !line.contains("ClientA") && !line.contains("ClientB") && !line.contains("ClientC") {
                    return Err(SensitiveDataViolation {
                        file_path: file_path_str.clone(),
                        line_number,
                        line_content: line.to_string(),
                        issue_description: violation.clone(),
                    });
                }
                
                if line.contains("workspace/Clients") && !line.contains("/path/to/client/repositories") {
                    return Err(SensitiveDataViolation {
                        file_path: file_path_str.clone(),
                        line_number,
                        line_content: line.to_string(),
                        issue_description: violation.clone(),
                    });
                }
            }
        }
    }

    Ok(())
}
