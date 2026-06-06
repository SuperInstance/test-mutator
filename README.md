# test-mutator

A Rust mutation testing library. Generate code mutants, run your test suite against each one, and find out where your tests are weak.

## What is Mutation Testing?

Mutation testing injects small faults (mutations) into your source code and checks whether your test suite catches them. If a mutant **survives** (tests still pass), it means your tests aren't thorough enough for that code path.

## Quick Start

```rust
use test_mutator::{MutationEngine, TestRunner, MutantReport};

// Generate all possible mutants for a crate
let engine = MutationEngine::new("/path/to/your-crate").with_all_sources();
let mutants = engine.generate();

// Run cargo test for each mutant
let runner = TestRunner::new("/path/to/your-crate");
let results = runner.run_all(&mutants);

// Build a report
let report = MutantReport::new("your-crate", mutants);
println!("{}", report.summary());
println!("{}", report.to_json());
```

## Mutations Supported

| Category | Mutations |
|----------|-----------|
| **Operators** | `+` ↔ `-`, `*` ↔ `/`, `&&` ↔ `||`, `&` ↔ `\|` |
| **Comparisons** | `>` ↔ `<`, `==` ↔ `!=`, `>=` ↔ `<=` |
| **Booleans** | `true` ↔ `false` |

## Core Types

- **`Mutant`** — A single code mutation with file, line, column, original/replacement text, and status
- **`RustMutator`** — Applies mutation patterns to Rust source code
- **`MutationEngine`** — Discovers `.rs` files and generates all possible mutants
- **`TestRunner`** — Runs `cargo test` for each mutant and records killed/survived/skipped
- **`MutationScore`** — Calculates the mutation score (killed / testable mutants)
- **`MutantReport`** — JSON report showing which mutants survived

## Example: CLI Runner

```bash
# Run against a crate (limit to first 20 mutants)
cargo run --example mutate -- ~/repos/my-crate --limit 20

# Full run (all mutants)
cargo run --example mutate -- ~/repos/my-crate
```

## Real Results: fleet-scanner

Ran 20 mutants against [fleet-scanner](https://github.com/SuperInstance/fleet-scanner):

```
Mutation Score: 100.00% (14 killed / 14 testable, 0 survived, 6 skipped)
✅ No surviving mutants — your test suite is solid!
```

6 mutants were skipped (compilation errors from mutations in pattern matching), and all 14 testable mutants were killed by the existing test suite.

## License

MIT
