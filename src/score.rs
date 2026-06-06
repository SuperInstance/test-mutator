use crate::mutant::{Mutant, MutantStatus};
use serde::{Deserialize, Serialize};

/// Mutation testing score: the fraction of mutants killed by the test suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationScore {
    /// Total mutants generated.
    pub total: usize,
    /// Mutants killed (tests failed).
    pub killed: usize,
    /// Mutants that survived (tests passed — indicates weak test coverage).
    pub survived: usize,
    /// Mutants skipped (e.g. compilation errors).
    pub skipped: usize,
    /// Mutation score as a percentage: killed / (total - skipped) * 100.
    pub score_percent: f64,
}

impl MutationScore {
    /// Calculate mutation score from a list of mutants.
    pub fn from_mutants(mutants: &[Mutant]) -> Self {
        let total = mutants.len();
        let killed = mutants.iter().filter(|m| m.status == MutantStatus::Killed).count();
        let survived = mutants.iter().filter(|m| m.status == MutantStatus::Survived).count();
        let skipped = mutants.iter().filter(|m| m.status == MutantStatus::Skipped).count();

        let testable = total - skipped;
        let score_percent = if testable > 0 {
            (killed as f64 / testable as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total,
            killed,
            survived,
            skipped,
            score_percent: (score_percent * 100.0).round() / 100.0,
        }
    }

    /// Calculate from raw counts.
    pub fn from_counts(total: usize, killed: usize, survived: usize, skipped: usize) -> Self {
        let testable = total - skipped;
        let score_percent = if testable > 0 {
            (killed as f64 / testable as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total,
            killed,
            survived,
            skipped,
            score_percent: (score_percent * 100.0).round() / 100.0,
        }
    }
}

impl std::fmt::Display for MutationScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Mutation Score: {:.2}% ({} killed / {} testable, {} survived, {} skipped)",
            self.score_percent, self.killed, self.total - self.skipped, self.survived, self.skipped
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutant::MutationKind;

    fn make_mutant(status: MutantStatus) -> Mutant {
        Mutant::new("test.rs", 1, 1, "+", "-", MutationKind::SwapOperator {
            from: "+".into(), to: "-".into(),
        }).with_status(status)
    }

    #[test]
    fn test_perfect_score() {
        let mutants: Vec<Mutant> = (0..5).map(|_| make_mutant(MutantStatus::Killed)).collect();
        let score = MutationScore::from_mutants(&mutants);
        assert_eq!(score.total, 5);
        assert_eq!(score.killed, 5);
        assert_eq!(score.survived, 0);
        assert!((score.score_percent - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_zero_score() {
        let mutants: Vec<Mutant> = (0..3).map(|_| make_mutant(MutantStatus::Survived)).collect();
        let score = MutationScore::from_mutants(&mutants);
        assert_eq!(score.killed, 0);
        assert_eq!(score.survived, 3);
        assert!((score.score_percent - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_mixed_score() {
        let mutants = vec![
            make_mutant(MutantStatus::Killed),
            make_mutant(MutantStatus::Killed),
            make_mutant(MutantStatus::Survived),
            make_mutant(MutantStatus::Skipped),
        ];
        let score = MutationScore::from_mutants(&mutants);
        assert_eq!(score.total, 4);
        assert_eq!(score.killed, 2);
        assert_eq!(score.survived, 1);
        assert_eq!(score.skipped, 1);
        // score = 2/3 * 100 = 66.67
        assert!((score.score_percent - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_from_counts() {
        let score = MutationScore::from_counts(10, 7, 2, 1);
        assert_eq!(score.total, 10);
        // 7/9 * 100 = 77.78
        assert!((score.score_percent - 77.78).abs() < 0.1);
    }

    #[test]
    fn test_empty_mutants() {
        let score = MutationScore::from_mutants(&[]);
        assert_eq!(score.total, 0);
        assert!((score.score_percent - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_display() {
        let score = MutationScore::from_counts(10, 8, 2, 0);
        let display = format!("{}", score);
        assert!(display.contains("80.00%"));
        assert!(display.contains("8 killed"));
    }
}
