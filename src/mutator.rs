use crate::mutant::{MutationKind, Mutant};

/// Applies mutation patterns to Rust source code.
pub struct RustMutator;

impl RustMutator {
    /// Operator swaps: `+` ↔ `-`, `*` ↔ `/`, `&&` ↔ `||`, `&` ↔ `|` (bitwise)
    const OPERATOR_SWAPS: &[(&str, &str)] = &[
        ("+", "-"),
        ("-", "+"),
        ("*", "/"),
        ("/", "*"),
        ("&&", "||"),
        ("||", "&&"),
        ("&", "|"),
        ("|", "&"),
    ];

    /// Comparison operator swaps: `>` ↔ `<`, `==` ↔ `!=`, `>=` ↔ `<=`
    const COMPARISON_SWAPS: &[(&str, &str)] = &[
        (">", "<"),
        ("<", ">"),
        ("==", "!="),
        ("!=", "=="),
        (">=", "<="),
        ("<=", ">="),
    ];

    /// Boolean literal swaps
    const BOOLEAN_SWAPS: &[(&str, &str)] = &[
        ("true", "false"),
        ("false", "true"),
    ];

    /// Generate all possible mutants for the given source file content.
    pub fn generate_mutants(file: &str, source: &str) -> Vec<Mutant> {
        let mut mutants = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!") {
                continue;
            }

            // Check comparison operators first (longer matches before shorter)
            for (from, to) in Self::COMPARISON_SWAPS {
                Self::find_occurrences(file, line_num, line, from, to, &mut mutants, |f, t| {
                    MutationKind::SwapComparison {
                        from: f.to_string(),
                        to: t.to_string(),
                    }
                });
            }

            // Check operator swaps
            for (from, to) in Self::OPERATOR_SWAPS {
                Self::find_occurrences(file, line_num, line, from, to, &mut mutants, |f, t| {
                    MutationKind::SwapOperator {
                        from: f.to_string(),
                        to: t.to_string(),
                    }
                });
            }

            // Check boolean swaps (whole-word only)
            for (from, to) in Self::BOOLEAN_SWAPS {
                Self::find_word_occurrences(file, line_num, line, from, to, &mut mutants);
            }
        }

        Self::dedup_mutants(mutants)
    }

    /// Apply a mutant to source code, returning the mutated source.
    pub fn apply_mutation(source: &str, mutant: &Mutant) -> String {
        let mut lines: Vec<String> = source.lines().map(|l| l.to_string()).collect();
        if mutant.line == 0 || mutant.line > lines.len() {
            return source.to_string();
        }
        let line_idx = mutant.line - 1;
        let line = &lines[line_idx];

        if let Some(replaced) = replace_at_column(line, &mutant.original, &mutant.replacement, mutant.column) {
            lines[line_idx] = replaced;
        }
        lines.join("\n")
    }

    fn find_occurrences<F>(
        file: &str,
        line_num: usize,
        line: &str,
        from: &str,
        to: &str,
        mutants: &mut Vec<Mutant>,
        kind_fn: F,
    ) where
        F: Fn(&str, &str) -> MutationKind,
    {
        let mut search_from = 0;

        while let Some(pos) = line[search_from..].find(from) {
            let abs_pos = search_from + pos;
            if !Self::is_inside_string(line, abs_pos) {
                let kind = kind_fn(from, to);
                mutants.push(Mutant::new(
                    file,
                    line_num,
                    abs_pos + 1,
                    from,
                    to,
                    kind,
                ));
            }
            search_from = abs_pos + from.len();
        }
    }

    fn find_word_occurrences(
        file: &str,
        line_num: usize,
        line: &str,
        from: &str,
        to: &str,
        mutants: &mut Vec<Mutant>,
    ) {
        let bytes = line.as_bytes();
        let mut search_from = 0;

        while let Some(pos) = line[search_from..].find(from) {
            let abs_pos = search_from + pos;
            let before_ok = abs_pos == 0 || !bytes[abs_pos - 1].is_ascii_alphanumeric();
            let after_idx = abs_pos + from.len();
            let after_ok = after_idx >= line.len() || !bytes[after_idx].is_ascii_alphanumeric();

            if before_ok && after_ok && !Self::is_inside_string(line, abs_pos) {
                mutants.push(Mutant::new(
                    file,
                    line_num,
                    abs_pos + 1,
                    from,
                    to,
                    MutationKind::SwapBoolean {
                        from: from.to_string(),
                        to: to.to_string(),
                    },
                ));
            }
            search_from = abs_pos + from.len();
        }
    }

    fn is_inside_string(line: &str, pos: usize) -> bool {
        let bytes = line.as_bytes();
        let mut in_string = false;
        let mut in_char = false;
        let mut i = 0usize;
        while i < pos && i < bytes.len() {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if bytes[i] == b'"' && !in_char {
                in_string = !in_string;
            }
            if bytes[i] == b'\'' && !in_string {
                in_char = !in_char;
            }
            i += 1;
        }
        in_string || in_char
    }

    fn dedup_mutants(mut mutants: Vec<Mutant>) -> Vec<Mutant> {
        let mut seen = std::collections::HashSet::new();
        mutants.retain(|m| seen.insert((m.file.clone(), m.line, m.column, m.original.clone())));
        mutants
    }
}

fn replace_at_column(line: &str, from: &str, to: &str, column: usize) -> Option<String> {
    let byte_idx = column.saturating_sub(1);
    let bytes = line.as_bytes();
    if byte_idx + from.len() > bytes.len() {
        return None;
    }
    if &bytes[byte_idx..byte_idx + from.len()] == from.as_bytes() {
        let mut result = line[..byte_idx].to_string();
        result.push_str(to);
        result.push_str(&line[byte_idx + from.len()..]);
        Some(result)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_plus_to_minus() {
        let source = "let x = a + b;";
        let mutants = RustMutator::generate_mutants("test.rs", source);
        assert!(mutants.iter().any(|m| m.original == "+" && m.replacement == "-"));
    }

    #[test]
    fn test_swap_true_to_false() {
        let source = "let flag = true;";
        let mutants = RustMutator::generate_mutants("test.rs", source);
        assert!(mutants.iter().any(|m| m.original == "true" && m.replacement == "false"));
    }

    #[test]
    fn test_swap_comparison_gt_lt() {
        let source = "if x > 0 {";
        let mutants = RustMutator::generate_mutants("test.rs", source);
        assert!(mutants.iter().any(|m| m.original == ">" && m.replacement == "<"));
    }

    #[test]
    fn test_swap_eq_neq() {
        let source = "if a == b {";
        let mutants = RustMutator::generate_mutants("test.rs", source);
        assert!(mutants.iter().any(|m| m.original == "==" && m.replacement == "!="));
    }

    #[test]
    fn test_apply_mutation() {
        let source = "let x = a + b;";
        let mutant = Mutant::new("test.rs", 1, 11, "+", "-", MutationKind::SwapOperator {
            from: "+".into(), to: "-".into(),
        });
        let result = RustMutator::apply_mutation(source, &mutant);
        assert_eq!(result, "let x = a - b;");
    }

    #[test]
    fn test_skip_comment_lines() {
        let source = "// This is a + comment\nlet x = a + b;";
        let mutants = RustMutator::generate_mutants("test.rs", source);
        assert!(mutants.iter().all(|m| m.line == 2));
    }

    #[test]
    fn test_skip_string_literals() {
        let source = r#"let s = "a + b";"#;
        let mutants = RustMutator::generate_mutants("test.rs", source);
        assert!(mutants.is_empty());
    }

    #[test]
    fn test_multiple_mutations_per_line() {
        let source = "let x = a + b + c;";
        let mutants = RustMutator::generate_mutants("test.rs", source);
        let plus_swaps: Vec<_> = mutants.iter().filter(|m| m.original == "+").collect();
        assert_eq!(plus_swaps.len(), 2);
    }

    #[test]
    fn test_boolean_word_boundary() {
        let source = "let is_trueflag = true;";
        let mutants = RustMutator::generate_mutants("test.rs", source);
        let bool_mutants: Vec<_> = mutants.iter()
            .filter(|m| matches!(m.kind, MutationKind::SwapBoolean { .. }))
            .collect();
        assert_eq!(bool_mutants.len(), 1);
        assert_eq!(bool_mutants[0].column, 19);
    }

    #[test]
    fn test_and_or_swap() {
        let source = "if a && b {";
        let mutants = RustMutator::generate_mutants("test.rs", source);
        assert!(mutants.iter().any(|m| m.original == "&&" && m.replacement == "||"));
    }

    #[test]
    fn test_ge_le_swap() {
        let source = "if x >= 10 {";
        let mutants = RustMutator::generate_mutants("test.rs", source);
        assert!(mutants.iter().any(|m| m.original == ">=" && m.replacement == "<="));
    }
}
