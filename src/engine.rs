use crate::mutant::Mutant;
use crate::mutator::RustMutator;
use std::path::{Path, PathBuf};

/// Generates all possible mutants for one or more Rust source files.
pub struct MutationEngine {
    /// Root directory of the project being mutated.
    root: PathBuf,
    /// Specific files to mutate (relative to root). If empty, all `.rs` files in `src/` are used.
    files: Vec<PathBuf>,
}

impl MutationEngine {
    /// Create an engine targeting the given project root.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            files: Vec::new(),
        }
    }

    /// Restrict mutation to specific files (relative paths under the project root).
    pub fn with_files(mut self, files: Vec<impl AsRef<Path>>) -> Self {
        self.files = files.into_iter().map(|f| f.as_ref().to_path_buf()).collect();
        self
    }

    /// Auto-discover `.rs` files under `src/` relative to root.
    pub fn with_all_sources(mut self) -> Self {
        let src_dir = self.root.join("src");
        if src_dir.is_dir() {
            self.files = discover_rust_files(&src_dir);
        }
        self
    }

    /// Generate all mutants for the configured files.
    pub fn generate(&self) -> Vec<Mutant> {
        let files = if self.files.is_empty() {
            discover_rust_files(&self.root.join("src"))
        } else {
            self.files.clone()
        };

        let mut all_mutants = Vec::new();
        for rel_path in &files {
            let full_path = self.root.join(rel_path);
            if let Ok(source) = std::fs::read_to_string(&full_path) {
                let file_str = rel_path.to_string_lossy();
                let mutants = RustMutator::generate_mutants(&file_str, &source);
                all_mutants.extend(mutants);
            }
        }
        all_mutants
    }

    /// Generate mutants for an in-memory source string, treating it as the given file name.
    pub fn generate_for_source(file: &str, source: &str) -> Vec<Mutant> {
        RustMutator::generate_mutants(file, source)
    }

    /// Return the project root.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// Recursively discover `.rs` files, returning paths relative to the given base.
fn discover_rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(discover_rust_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                if let Ok(rel) = path.strip_prefix(dir) {
                    // Prepend "src/" to make it relative to project root
                    result.push(PathBuf::from("src").join(rel));
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_engine_discovers_rs_files() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
        fs::write(src.join("main.rs"), "fn main() { println!(\"hello\"); }").unwrap();

        let engine = MutationEngine::new(dir.path()).with_all_sources();
        let mutants = engine.generate();
        // Should find at least the `+` → `-` mutation in lib.rs
        assert!(!mutants.is_empty(), "Should find at least one mutant");
        assert!(mutants.iter().any(|m| m.file.contains("lib.rs")));
    }

    #[test]
    fn test_engine_with_specific_files() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
        fs::write(src.join("other.rs"), "pub fn x() -> bool { true }").unwrap();

        let engine = MutationEngine::new(dir.path())
            .with_files(vec!["src/lib.rs"]);
        let mutants = engine.generate();
        assert!(mutants.iter().all(|m| m.file == "src/lib.rs"));
    }

    #[test]
    fn test_generate_for_source() {
        let source = "fn f(x: i32) -> bool { x > 0 }";
        let mutants = MutationEngine::generate_for_source("test.rs", source);
        assert!(!mutants.is_empty());
    }
}
