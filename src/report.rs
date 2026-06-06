use crate::mutant::{Mutant, MutantStatus};
use crate::score::MutationScore;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A full mutation testing report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutantReport {
    /// Timestamp of the report.
    pub timestamp: String,
    /// The crate that was tested.
    pub target_crate: String,
    /// All mutants with their statuses.
    pub mutants: Vec<Mutant>,
    /// Aggregated mutation score.
    pub score: MutationScore,
    /// Mutants that survived (the ones to worry about).
    pub survived: Vec<SurvivedMutant>,
}

/// A mutant that survived — with enough context to locate and fix the weak test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurvivedMutant {
    pub id: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub kind: String,
    pub original: String,
    pub replacement: String,
}

impl MutantReport {
    /// Build a report from a completed mutation run.
    pub fn new(target_crate: &str, mutants: Vec<Mutant>) -> Self {
        let score = MutationScore::from_mutants(&mutants);
        let survived: Vec<SurvivedMutant> = mutants
            .iter()
            .filter(|m| m.status == MutantStatus::Survived)
            .map(|m| SurvivedMutant {
                id: m.id.clone(),
                file: m.file.clone(),
                line: m.line,
                column: m.column,
                kind: format!("{}", m.kind),
                original: m.original.clone(),
                replacement: m.replacement.clone(),
            })
            .collect();

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            timestamp,
            target_crate: target_crate.to_string(),
            mutants,
            score,
            survived,
        }
    }

    /// Render the report as pretty-printed JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }

    /// Render a human-readable summary.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("═══════════════════════════════════════════"));
        lines.push(format!("Mutation Report: {}", self.target_crate));
        lines.push(format!("═══════════════════════════════════════════"));
        lines.push(format!("{}", self.score));
        lines.push(String::new());

        if self.survived.is_empty() {
            lines.push("✅ No surviving mutants — your test suite is solid!".to_string());
        } else {
            lines.push(format!("⚠️  {} surviving mutant(s) — weak test areas:", self.survived.len()));
            lines.push(String::new());
            for s in &self.survived {
                lines.push(format!(
                    "  • {}:{}:{} — {} ({} → {})",
                    s.file, s.line, s.column, s.kind, s.original, s.replacement
                ));
            }
            lines.push(String::new());
            lines.push("These mutations went undetected. Consider adding tests that cover:".to_string());
            lines.push("  - The specific branches/operators listed above".to_string());
            lines.push("  - Edge cases that would catch swapped operators".to_string());
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutant::MutationKind;

    fn make_mutant(status: MutantStatus, file: &str, line: usize) -> Mutant {
        Mutant::new(file, line, 10, "+", "-", MutationKind::SwapOperator {
            from: "+".into(), to: "-".into(),
        }).with_status(status)
    }

    #[test]
    fn test_report_no_survivors() {
        let mutants = vec![
            make_mutant(MutantStatus::Killed, "a.rs", 1),
            make_mutant(MutantStatus::Killed, "a.rs", 5),
        ];
        let report = MutantReport::new("test-crate", mutants);
        assert!(report.survived.is_empty());
        assert_eq!(report.score.killed, 2);
        assert!(report.summary().contains("No surviving mutants"));
    }

    #[test]
    fn test_report_with_survivors() {
        let mutants = vec![
            make_mutant(MutantStatus::Killed, "a.rs", 1),
            make_mutant(MutantStatus::Survived, "b.rs", 3),
        ];
        let report = MutantReport::new("test-crate", mutants);
        assert_eq!(report.survived.len(), 1);
        assert_eq!(report.survived[0].file, "b.rs");
        assert!(report.summary().contains("surviving mutant"));
    }

    #[test]
    fn test_report_json_output() {
        let mutants = vec![make_mutant(MutantStatus::Killed, "a.rs", 1)];
        let report = MutantReport::new("test-crate", mutants);
        let json = report.to_json();
        assert!(json.contains("\"target_crate\": \"test-crate\""));
        assert!(json.contains("\"killed\": 1"));
    }
}
