use serde::{Deserialize, Serialize};
use std::fmt;

/// The kind of mutation applied to source code.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MutationKind {
    /// Swap binary operator: `+` → `-`, `-` → `+`, `*` → `/`, `/` → `*`
    SwapOperator { from: String, to: String },
    /// Change boolean constant: `true` → `false`, `false` → `true`
    SwapBoolean { from: String, to: String },
    /// Change comparison operator: `>` → `<`, `<` → `>`, `==` → `!=`, `!=` → `==`,
    /// `>=` → `<=`, `<=` → `>=`
    SwapComparison { from: String, to: String },
    /// Negate a condition by wrapping in `!()`
    NegateCondition,
}

impl fmt::Display for MutationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MutationKind::SwapOperator { from, to } => write!(f, "operator: {from} → {to}"),
            MutationKind::SwapBoolean { from, to } => write!(f, "boolean: {from} → {to}"),
            MutationKind::SwapComparison { from, to } => write!(f, "comparison: {from} → {to}"),
            MutationKind::NegateCondition => write!(f, "negate condition"),
        }
    }
}

/// Whether a mutant was killed by the test suite, survived, or was skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutantStatus {
    /// The test suite failed — the mutant was detected.
    Killed,
    /// The test suite passed — the mutant went undetected (weak tests).
    Survived,
    /// The mutant was not tested (e.g. compilation error).
    Skipped,
    /// Not yet run.
    Pending,
}

impl fmt::Display for MutantStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MutantStatus::Killed => write!(f, "killed"),
            MutantStatus::Survived => write!(f, "survived"),
            MutantStatus::Skipped => write!(f, "skipped"),
            MutantStatus::Pending => write!(f, "pending"),
        }
    }
}

/// A single mutation applied to a source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mutant {
    /// Unique identifier for this mutant.
    pub id: String,
    /// Path to the source file being mutated (relative).
    pub file: String,
    /// 1-indexed line number where the mutation applies.
    pub line: usize,
    /// 1-indexed column (byte offset from line start) where the original text begins.
    pub column: usize,
    /// The original text that gets replaced.
    pub original: String,
    /// The replacement text.
    pub replacement: String,
    /// The category of mutation.
    pub kind: MutationKind,
    /// Result of running the test suite against this mutant.
    pub status: MutantStatus,
}

impl Mutant {
    /// Create a new mutant with pending status and a generated UUID.
    pub fn new(
        file: impl Into<String>,
        line: usize,
        column: usize,
        original: impl Into<String>,
        replacement: impl Into<String>,
        kind: MutationKind,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            file: file.into(),
            line,
            column,
            original: original.into(),
            replacement: replacement.into(),
            kind,
            status: MutantStatus::Pending,
        }
    }

    /// Mark this mutant with a status.
    pub fn with_status(mut self, status: MutantStatus) -> Self {
        self.status = status;
        self
    }
}

impl fmt::Display for Mutant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}:{}:{} — {} ({} → {})",
            self.status, self.file, self.line, self.column, self.kind, self.original, self.replacement
        )
    }
}
