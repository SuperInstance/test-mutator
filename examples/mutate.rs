//! Run mutation testing on a target crate.
//!
//! Usage: cargo run --example mutate -- /path/to/target-crate [--limit N]

use std::env;
use std::fs;
use test_mutator::{MutationEngine, MutantReport, TestRunner};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mutate <crate-path> [--limit N]");
        std::process::exit(1);
    }

    let crate_path = &args[1];
    let mut limit: Option<usize> = None;
    for i in 2..args.len() {
        if args[i] == "--limit" && i + 1 < args.len() {
            limit = args[i + 1].parse().ok();
        }
    }

    println!("🔍 Generating mutants for: {}", crate_path);

    let engine = MutationEngine::new(crate_path).with_all_sources();
    let mut mutants = engine.generate();

    println!("   Found {} possible mutants", mutants.len());

    if mutants.is_empty() {
        eprintln!("No mutants found. Is this a Rust crate with source in src/?");
        std::process::exit(1);
    }

    if let Some(limit) = limit {
        mutants.truncate(limit);
        println!("   (Limited to {} mutants)", mutants.len());
    }

    println!("\n🧪 Running mutation tests...\n");

    let runner = TestRunner::new(crate_path).with_timeout(60);
    let results = runner.run_all(&mutants);

    // Update mutants with results
    for (mutant, result) in mutants.iter_mut().zip(results) {
        mutant.status = result.status;
        let emoji = match mutant.status {
            test_mutator::MutantStatus::Killed => "💀",
            test_mutator::MutantStatus::Survived => " survivor",
            test_mutator::MutantStatus::Skipped => "⏭️ ",
            test_mutator::MutantStatus::Pending => "⏳",
        };
        println!(
            "  {} [{}] {}:{}:{} — {} → {}",
            emoji, mutant.status, mutant.file, mutant.line, mutant.column,
            mutant.original, mutant.replacement
        );
    }

    let report = MutantReport::new(crate_path, mutants);

    println!("\n{}", report.summary());

    // Write JSON report
    let json_path = format!("{}/mutation-report.json", crate_path);
    if let Err(e) = fs::write(&json_path, report.to_json()) {
        eprintln!("Failed to write report: {}", e);
    } else {
        println!("\n📄 Full report saved to: {}", json_path);
    }
}
