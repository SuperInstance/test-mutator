# Test Mutator

**Test Mutator** is a mutation testing library for Rust that automatically generates code mutants, runs your test suite against each one, and calculates a **mutation score** — the fraction of injected faults your tests actually catch. It reveals the difference between "100% line coverage" and "100% fault detection."

## Why It Matters

Coverage tools measure which lines your tests execute, but executing a line isn't the same as *verifying* it. Mutation testing goes further: it deliberately introduces bugs (changing `+` to `−`, `>` to `>=`, `true` to `false`) and checks whether your tests fail. If a test suite passes despite a mutant, your tests are inadequate — they would miss that exact bug in production. Studies show that even projects with 100% line coverage typically score 60–80% on mutation testing. Test Mutator automates this analysis for Rust projects, providing per-function mutation scores and detailed reports of surviving mutants — the bugs your tests would miss.

## How It Works

### Mutation Operators

Test Mutator applies systematic code transformations:

| Mutation | Example | What It Tests |
|----------|---------|---------------|
| Arithmetic operator | `a + b → a − b` | Boundary arithmetic |
| Comparison operator | `x > 0 → x >= 0` | Off-by-one detection |
| Logical operator | `a && b → a \|\| b` | Short-circuit sensitivity |
| Boolean literal | `true → false` | Default/edge case coverage |
| Constant mutation | `0 → 1` | Magic number tests |
| Statement deletion | `foo(); → ` | Side-effect awareness |
| Return mutation | `Ok(x) → Err(())` | Error path coverage |

### Mutation Engine Pipeline

```
Source code → Parse → AST → Apply mutation → Generate mutant → Run tests → Score
```

1. **Parse**: `RustMutator` reads source files and builds an AST
2. **Generate mutants**: Each mutation operator produces a `Mutant` struct containing the modified source, location, and kind
3. **Compile and test**: `TestRunner` compiles each mutant and runs the test suite
4. **Score**: `MutationScore` aggregates results

For M mutants across N tests:
- Compilation: O(M) compile cycles (dominant cost)
- Test execution: O(M × T) where T = test suite runtime
- Total: O(M × T) — typically minutes for small crates

### Mutation Score

```
Mutation Score = Killed / (Killed + Survived + Timeout)

Killed   = test suite failed (mutant detected)
Survived = test suite passed (mutant NOT detected — bad!)
Timeout  = test suite hung (mutant caused infinite loop)
```

A score of 100% means every injected fault was caught. 80%+ is considered strong. Below 50% means the test suite is mostly theater.

### Report Generation

The `MutantReport` struct provides:
- Per-mutation-kind survival rates
- Per-file mutation scores
- List of surviving mutants with source locations
- Equivalent mutant detection (mutants that produce identical behavior)

## Quick Start

```rust
use test_mutator::{MutationEngine, RustMutator, TestRunner};

fn main() {
    let mut engine = MutationEngine::new("src/", "tests/");
    let mutants = engine.generate_mutants();

    println!("Generated {} mutants", mutants.len());

    let runner = TestRunner::new();
    let results = runner.run_all(&mutants);

    let score = results.mutation_score();
    println!("Mutation Score: {:.1}%", score * 100.0);
    println!("Killed: {}, Survived: {}, Timeout: {}",
        results.killed, results.survived, results.timeout);

    for surviving in results.surviving_mutants() {
        println!("SURVIVED: {} at {}:{}",
            surviving.kind, surviving.file, surviving.line);
    }
}
```

```bash
cargo build
cargo run -- --source src/ --tests tests/
```

## API

| Type | Method | Description |
|------|--------|-------------|
| `RustMutator` | `parse(path)`, `mutate() → Vec<Mutant>` | Source code mutation |
| `MutationEngine` | `new(src_dir, test_dir)` | Orchestrates mutation |
| `MutationEngine` | `generate_mutants() → Vec<Mutant>` | Produce all mutants |
| `TestRunner` | `run_all(mutants) → Results` | Compile + test each mutant |
| `MutationScore` | `killed / survived / timeout` | Score breakdown |
| `MutantReport` | `per_file()`, `surviving()` | Detailed report |
| `Mutant` | `kind`, `file`, `line`, `diff` | Single mutation |
| `MutationKind` | enum of all mutation types | Categorization |

## Architecture Notes

Test Mutator is the quality-assurance arm of η (eta) in the γ + η = C framework. Where η normally *eliminates* code bugs, Test Mutator *measures* how good your η processes are — it reveals the gaps where bugs would survive. A high mutation score means your η (testing, review, elimination) is effective. A low score means your C (competence) is fragile — it depends on untested assumptions. By systematically injecting faults and measuring detection, Test Mutator quantifies the gap between perceived and actual test quality. See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

1. DeMillo, R. A., Lipton, R. J., & Sayward, F. G. (1978). "Hints on Test Data Selection: Help for the Struggling Programmer." *IEEE Transactions on Software Engineering*, 4(1), 34–41. — Original mutation testing paper.
2. Andrews, J. H., et al. (2005). "Is Mutation an Appropriate Tool for Testing Experiments?" *ICSE*. — Empirical validation of mutation testing.
3. Papadakis, M., et al. (2019). "Mutation Testing Advances: An Analysis and Survey." *Advances in Computers*, 112, 275–378.

## License

MIT
