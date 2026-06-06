use crate::mutant::{Mutant, MutantStatus};
use crate::mutator::RustMutator;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Runs `cargo test` against each mutant and records whether the mutant was killed.
pub struct TestRunner {
    /// Path to the crate being tested.
    crate_path: PathBuf,
    /// Timeout in seconds for each test run.
    timeout_secs: u64,
    /// Whether to show cargo output.
    verbose: bool,
}

/// Result of running a single mutant through the test suite.
#[derive(Debug, Clone)]
pub struct TestRunResult {
    pub mutant_id: String,
    pub status: MutantStatus,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

impl TestRunner {
    /// Create a new test runner for the given crate path.
    pub fn new(crate_path: impl AsRef<Path>) -> Self {
        Self {
            crate_path: crate_path.as_ref().to_path_buf(),
            timeout_secs: 120,
            verbose: false,
        }
    }

    /// Set the timeout per test run.
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Enable verbose output.
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Run all mutants against the test suite.
    ///
    /// For each mutant:
    /// 1. Backs up the original source file
    /// 2. Applies the mutation
    /// 3. Runs `cargo test`
    /// 4. Restores the original source
    /// 5. Records whether the mutant was killed (test failed) or survived (test passed)
    pub fn run_all(&self, mutants: &[Mutant]) -> Vec<TestRunResult> {
        let mut results = Vec::new();
        for mutant in mutants {
            results.push(self.run_one(mutant));
        }
        results
    }

    /// Run a single mutant through the test suite.
    pub fn run_one(&self, mutant: &Mutant) -> TestRunResult {
        let file_path = self.crate_path.join(&mutant.file);

        // Read original source
        let original_source = match std::fs::read_to_string(&file_path) {
            Ok(s) => s,
            Err(e) => {
                return TestRunResult {
                    mutant_id: mutant.id.clone(),
                    status: MutantStatus::Skipped,
                    stdout: String::new(),
                    stderr: format!("Failed to read {}: {}", file_path.display(), e),
                    duration_ms: 0,
                };
            }
        };

        // Apply mutation
        let mutated_source = RustMutator::apply_mutation(&original_source, mutant);

        // Write mutated source
        if let Err(e) = std::fs::write(&file_path, &mutated_source) {
            return TestRunResult {
                mutant_id: mutant.id.clone(),
                status: MutantStatus::Skipped,
                stdout: String::new(),
                stderr: format!("Failed to write {}: {}", file_path.display(), e),
                duration_ms: 0,
            };
        }

        // Run cargo test
        let start = std::time::Instant::now();
        let output = Command::new("cargo")
            .args(["test", "--quiet", "--color=never"])
            .current_dir(&self.crate_path)
            .env("CARGO_TERM_COLOR", "never")
            .output();
        let duration_ms = start.elapsed().as_millis() as u64;

        // Restore original source
        let _ = std::fs::write(&file_path, &original_source);

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();

                // If cargo test exits with non-zero, a test failed → mutant killed
                // But we need to distinguish compilation errors from test failures
                let status = if out.status.success() {
                    MutantStatus::Survived
                } else if stderr.contains("error[") && !stderr.contains("test result:") {
                    // Compilation error — skip this mutant
                    MutantStatus::Skipped
                } else {
                    MutantStatus::Killed
                };

                TestRunResult {
                    mutant_id: mutant.id.clone(),
                    status,
                    stdout,
                    stderr,
                    duration_ms,
                }
            }
            Err(e) => TestRunResult {
                mutant_id: mutant.id.clone(),
                status: MutantStatus::Skipped,
                stdout: String::new(),
                stderr: format!("Failed to run cargo test: {}", e),
                duration_ms: duration_ms,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let runner = TestRunner::new("/tmp/fake");
        assert_eq!(runner.crate_path, PathBuf::from("/tmp/fake"));
    }

    #[test]
    fn test_runner_with_timeout() {
        let runner = TestRunner::new("/tmp/fake").with_timeout(30);
        assert_eq!(runner.timeout_secs, 30);
    }
}
