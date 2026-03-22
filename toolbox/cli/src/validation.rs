//! Architecture validation utilities
//!
//! This module validates Solana program architecture against AGENTS.md guidelines,
//! checking for required files, proper separation of concerns, and code compliance.
//!
//! # Examples
//!
//! ```no_run
//! use solana_toolbox::validation::{ValidationReport, validate_required_files};
//!
//! let mut report = ValidationReport::new();
//! validate_required_files(&mut report).unwrap();
//!
//! if report.has_errors() {
//!     eprintln!("Validation failed!");
//! }
//! report.print();
//! ```

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Required files for architecture compliance
const REQUIRED_FILES: &[&str] = &[
    "src/entrypoint.rs",
    "src/processor.rs",
    "src/instructions.rs",
    "src/state.rs",
    "src/errors.rs",
    "client/instructions.rs",
    "client/sender.rs",
    "client/solana_ctx.rs",
    "client/client.rs",
    "AGENTS.md",
    "Cargo.toml",
];

/// Validation report
///
/// Collects errors, warnings, and successes during architecture validation.
///
/// # Examples
///
/// ```
/// use solana_toolbox::validation::ValidationReport;
///
/// let mut report = ValidationReport::new();
/// report.add_success("File found".to_string());
/// report.add_warning("Optional file missing".to_string());
/// report.add_error("Required file missing".to_string());
///
/// assert!(report.has_errors());
/// report.print();
/// ```
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub successes: Vec<String>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            successes: Vec::new(),
        }
    }

    pub fn add_error(&mut self, message: String) {
        self.errors.push(message);
    }

    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }

    pub fn add_success(&mut self, message: String) {
        self.successes.push(message);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn print(&self) {
        // Print successes
        for success in &self.successes {
            println!("  {} {}", "✅".green(), success);
        }

        // Print warnings
        for warning in &self.warnings {
            println!("  {} {}", "⚠️".yellow(), warning);
        }

        // Print errors
        for error in &self.errors {
            println!("  {} {}", "❌".red(), error);
        }

        println!();
        println!("{}", "━".repeat(50).bright_black());

        if self.has_errors() {
            println!(
                "{} Failed with {} error(s) and {} warning(s)",
                "❌".red(),
                self.errors.len(),
                self.warnings.len()
            );
        } else if !self.warnings.is_empty() {
            println!(
                "{} Passed with {} warning(s)",
                "✅".green(),
                self.warnings.len()
            );
        } else {
            println!("{} All checks passed!", "✅".green());
        }
    }
}

/// Validate required files exist
///
/// Checks that all required files for AGENTS.md compliance are present.
///
/// # Arguments
///
/// * `report` - Mutable reference to validation report for collecting results
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::validation::{ValidationReport, validate_required_files};
///
/// let mut report = ValidationReport::new();
/// validate_required_files(&mut report).unwrap();
/// ```
pub fn validate_required_files(report: &mut ValidationReport) -> Result<()> {
    println!("{}", "📂 Checking required files...".bright_blue());

    for file_path in REQUIRED_FILES {
        let path = Path::new(file_path);
        if path.exists() {
            report.add_success(file_path.to_string());
        } else {
            report.add_error(format!("Missing required file: {}", file_path));
        }
    }

    Ok(())
}

/// Validate client layer separation of concerns
pub fn validate_client_separation(report: &mut ValidationReport) -> Result<()> {
    println!("{}", "🔬 Checking separation of concerns...".bright_blue());

    let client_instructions_path = "client/instructions.rs";
    if Path::new(client_instructions_path).exists() {
        let content = fs::read_to_string(client_instructions_path)
            .context("Failed to read client/instructions.rs")?;

        // Check for side effects (should be pure builders)
        let has_send = content.contains(".send_transaction")
            || content.contains("execute")
            || content.contains("RpcClient");

        if has_send {
            report.add_warning(
                "client/instructions.rs may contain side effects (should be pure builders)"
                    .to_string(),
            );
        } else {
            report.add_success("client/instructions.rs appears pure (no side effects)".to_string());
        }
    }

    Ok(())
}

/// Validate Borsh serialization consistency
pub fn validate_borsh_usage(report: &mut ValidationReport) -> Result<()> {
    println!("{}", "📦 Checking Borsh serialization consistency...".bright_blue());

    let instructions_path = "src/instructions.rs";
    let state_path = "src/state.rs";

    let mut has_borsh = true;

    if Path::new(instructions_path).exists() {
        let content = fs::read_to_string(instructions_path)?;
        if !content.contains("borsh") {
            report.add_error("src/instructions.rs missing Borsh imports".to_string());
            has_borsh = false;
        }
    }

    if Path::new(state_path).exists() {
        let content = fs::read_to_string(state_path)?;
        if !content.contains("borsh") {
            report.add_error("src/state.rs missing Borsh imports".to_string());
            has_borsh = false;
        }
    }

    if has_borsh {
        report.add_success("Borsh imports found in instruction and state modules".to_string());
    }

    Ok(())
}

/// Validate error handling patterns
pub fn validate_error_handling(report: &mut ValidationReport) -> Result<()> {
    println!("{}", "⚠️  Checking error handling patterns...".bright_blue());

    let errors_path = "src/errors.rs";

    if Path::new(errors_path).exists() {
        let content = fs::read_to_string(errors_path)?;

        if content.contains("thiserror::Error") {
            report.add_success("Using thiserror for domain errors".to_string());
        } else {
            report.add_warning("Consider using thiserror for error handling".to_string());
        }
    } else {
        report.add_error("Missing src/errors.rs".to_string());
    }

    Ok(())
}

/// Validate documentation coverage (basic check)
pub fn validate_documentation(report: &mut ValidationReport) -> Result<()> {
    println!("{}", "📚 Checking documentation coverage...".bright_blue());

    let src_files = vec![
        "src/instructions.rs",
        "src/processor.rs",
        "src/state.rs",
        "src/errors.rs",
    ];

    let mut total_lines = 0;
    let mut doc_lines = 0;

    for file_path in src_files {
        if let Ok(content) = fs::read_to_string(file_path) {
            for line in content.lines() {
                total_lines += 1;
                if line.trim_start().starts_with("///") || line.trim_start().starts_with("//!") {
                    doc_lines += 1;
                }
            }
        }
    }

    if total_lines > 0 {
        let coverage = (doc_lines as f64 / total_lines as f64) * 100.0;
        report.add_success(format!("Documentation coverage: ~{:.0}%", coverage));
    }

    Ok(())
}

/// Run all validation checks
pub fn validate_architecture() -> Result<ValidationReport> {
    let mut report = ValidationReport::new();

    validate_required_files(&mut report)?;
    validate_client_separation(&mut report)?;
    validate_borsh_usage(&mut report)?;
    validate_error_handling(&mut report)?;
    validate_documentation(&mut report)?;

    Ok(report)
}
