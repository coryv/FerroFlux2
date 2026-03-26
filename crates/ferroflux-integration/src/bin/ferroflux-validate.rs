use ferroflux_integration::{DefinitionRegistry, Severity};
use std::path::PathBuf;
use std::process;

fn main() {
    let path: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("platforms"));

    if !path.exists() {
        eprintln!("error: path does not exist: {}", path.display());
        process::exit(2);
    }

    let mut registry = DefinitionRegistry::default();
    let result = match registry.load_from_dir(&path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(2);
        }
    };

    let total_files =
        registry.definitions.len() + registry.platforms.len();
    let error_count = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warning_count = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    if result.diagnostics.is_empty() {
        println!("All {total_files} file(s) OK — no issues found.");
        process::exit(0);
    }

    // Group diagnostics by file for readable output
    let mut by_file: std::collections::BTreeMap<String, Vec<_>> =
        std::collections::BTreeMap::new();
    let mut no_file: Vec<_> = vec![];

    for diag in &result.diagnostics {
        match &diag.file {
            Some(f) => by_file
                .entry(f.display().to_string())
                .or_default()
                .push(diag),
            None => no_file.push(diag),
        }
    }

    for (file, diags) in &by_file {
        println!("{file}");
        for d in diags {
            let level = match d.severity {
                Severity::Error => "  ERROR",
                Severity::Warning => "  WARN ",
            };
            println!("{level} [{}] {}", d.rule, d.message);
        }
        println!();
    }

    for d in &no_file {
        let level = match d.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN ",
        };
        println!("{level} [{}] {}", d.rule, d.message);
    }

    println!(
        "{total_files} file(s) checked, {error_count} error(s), {warning_count} warning(s)"
    );

    if error_count > 0 {
        process::exit(1);
    }
}
