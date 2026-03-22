//! Code optimization analysis utilities
//!
//! This module analyzes Solana program code to find optimization opportunities
//! in account size, compute units, memory usage, and more.
//!
//! # Categories
//!
//! - **Account Size**: Reduce account data layout
//! - **Compute Units**: Minimize instruction complexity
//! - **Memory**: Stack vs heap allocation
//! - **Serialization**: Efficient Borsh usage
//! - **Account Validation**: Early validation, fail fast
//! - **Error Handling**: Reduce error type size
//! - **Documentation**: Missing docs
//!
//! # Examples
//!
//! ```no_run
//! use solana_toolbox::optimization::analyze_file;
//!
//! let optimizations = analyze_file("src/processor.rs").unwrap();
//! for opt in optimizations {
//!     println!("{:?}: {}", opt.category, opt.description);
//! }
//! ```

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use regex::Regex;

/// Optimization opportunity
///
/// Represents a potential code optimization with category, location, and suggestion.
///
/// # Examples
///
/// ```
/// use solana_toolbox::optimization::{Optimization, OptimizationCategory, Priority};
///
/// let opt = Optimization {
///     category: OptimizationCategory::ComputeUnits,
///     file: "src/processor.rs".to_string(),
///     line: 42,
///     description: "Complex nested loop".to_string(),
///     suggestion: "Consider caching intermediate values".to_string(),
///     priority: Priority::High,
/// };
/// ```
#[derive(Debug)]
pub struct Optimization {
    pub category: OptimizationCategory,
    pub file: String,
    pub line: usize,
    pub description: String,
    pub suggestion: String,
    pub priority: Priority,
}

/// Optimization category
///
/// Classifies optimization opportunities by domain.
///
/// # Examples
///
/// ```
/// use solana_toolbox::optimization::OptimizationCategory;
///
/// let category = OptimizationCategory::ComputeUnits;
/// assert_eq!(category.to_string(), "Compute Units");
/// ```
#[derive(Debug, PartialEq)]
pub enum OptimizationCategory {
    AccountSize,
    ComputeUnits,
    Memory,
    Serialization,
    AccountValidation,
    ErrorHandling,
    Documentation,
}

/// Priority level for optimizations
///
/// # Examples
///
/// ```
/// use solana_toolbox::optimization::Priority;
///
/// assert!(Priority::High > Priority::Medium);
/// assert!(Priority::Medium > Priority::Low);
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for OptimizationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccountSize => write!(f, "Account Size"),
            Self::ComputeUnits => write!(f, "Compute Units"),
            Self::Memory => write!(f, "Memory"),
            Self::Serialization => write!(f, "Serialization"),
            Self::AccountValidation => write!(f, "Account Validation"),
            Self::ErrorHandling => write!(f, "Error Handling"),
            Self::Documentation => write!(f, "Documentation"),
        }
    }
}

/// Analyze the codebase for optimization opportunities
pub fn analyze_codebase() -> Result<Vec<Optimization>> {
    let mut optimizations = Vec::new();

    // Analyze source files
    let src_files = vec![
        "src/processor.rs",
        "src/instructions.rs",
        "src/state.rs",
        "src/errors.rs",
        "src/entrypoint.rs",
    ];

    for file_path in src_files {
        if Path::new(file_path).exists() {
            let content = fs::read_to_string(file_path)
                .with_context(|| format!("Failed to read {}", file_path))?;
            
            analyze_file(file_path, &content, &mut optimizations)?;
        }
    }

    // Sort by priority
    optimizations.sort_by(|a, b| b.priority.cmp(&a.priority));

    Ok(optimizations)
}

/// Analyze a single file for optimization opportunities
fn analyze_file(file_path: &str, content: &str, optimizations: &mut Vec<Optimization>) -> Result<()> {
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let line_num = line_num + 1; // 1-indexed

        // Check for Vec usage in state structs (account size optimization)
        if file_path.contains("state.rs") && line.contains("Vec<") {
            optimizations.push(Optimization {
                category: OptimizationCategory::AccountSize,
                file: file_path.to_string(),
                line: line_num,
                description: "Vec in state struct can cause variable-size accounts".to_string(),
                suggestion: "Consider fixed-size arrays or separate accounts for collections".to_string(),
                priority: Priority::High,
            });
        }

        // Check for .clone() in hot paths
        if file_path.contains("processor.rs") && line.contains(".clone()") {
            optimizations.push(Optimization {
                category: OptimizationCategory::Memory,
                file: file_path.to_string(),
                line: line_num,
                description: "Clone operation in processor may increase compute units".to_string(),
                suggestion: "Consider borrowing or using references instead".to_string(),
                priority: Priority::Medium,
            });
        }

        // Check for missing account ownership validation
        if file_path.contains("processor.rs") && line.contains("let ") && line.contains("_info") {
            // Look ahead for ownership check
            let next_10_lines = &lines[line_num.min(lines.len())..line_num.min(lines.len()) + 10];
            let has_owner_check = next_10_lines.iter().any(|l| {
                l.contains(".owner") && l.contains("program_id")
            });

            if !has_owner_check && line.contains("writable") {
                optimizations.push(Optimization {
                    category: OptimizationCategory::AccountValidation,
                    file: file_path.to_string(),
                    line: line_num,
                    description: "Writable account may be missing ownership validation".to_string(),
                    suggestion: "Add check: if account.owner != program_id { return Err(...) }".to_string(),
                    priority: Priority::High,
                });
            }
        }

        // Check for unwrap() usage (should use ? or proper error handling)
        if line.contains(".unwrap()") && !line.trim_start().starts_with("//") {
            optimizations.push(Optimization {
                category: OptimizationCategory::ErrorHandling,
                file: file_path.to_string(),
                line: line_num,
                description: "unwrap() can cause panic in production".to_string(),
                suggestion: "Replace with proper error handling using ? or match".to_string(),
                priority: Priority::High,
            });
        }

        // Check for large inline data
        if line.contains("[u8;") {
            let size_re = Regex::new(r"\[u8;\s*(\d+)\]").unwrap();
            if let Some(caps) = size_re.captures(line) {
                if let Some(size_str) = caps.get(1) {
                    if let Ok(size) = size_str.as_str().parse::<usize>() {
                        if size > 256 {
                            optimizations.push(Optimization {
                                category: OptimizationCategory::AccountSize,
                                file: file_path.to_string(),
                                line: line_num,
                                description: format!("Large byte array [u8; {}] increases account size", size),
                                suggestion: "Consider dynamic allocation or separate data account".to_string(),
                                priority: Priority::Medium,
                            });
                        }
                    }
                }
            }
        }

        // Check for missing documentation on pub items
        if line.trim().starts_with("pub ") && !line.contains("//") {
            // Check if previous line has doc comment
            let has_doc = if line_num > 1 {
                lines[line_num - 2].trim_start().starts_with("///")
            } else {
                false
            };

            if !has_doc {
                optimizations.push(Optimization {
                    category: OptimizationCategory::Documentation,
                    file: file_path.to_string(),
                    line: line_num,
                    description: "Public item missing documentation".to_string(),
                    suggestion: "Add /// doc comments following Rust API guidelines".to_string(),
                    priority: Priority::Low,
                });
            }
        }

        // Check for String in Solana programs (should use &str or fixed arrays)
        if (file_path.contains("state.rs") || file_path.contains("instructions.rs")) 
            && line.contains("String") 
            && !line.trim_start().starts_with("//") {
            optimizations.push(Optimization {
                category: OptimizationCategory::Serialization,
                file: file_path.to_string(),
                line: line_num,
                description: "String type can cause variable serialization size".to_string(),
                suggestion: "Consider fixed-size [u8; N] with UTF-8 validation".to_string(),
                priority: Priority::Medium,
            });
        }
    }

    Ok(())
}

/// Print optimization report
pub fn print_optimization_report(optimizations: &[Optimization]) {
    println!("{}", "═".repeat(50).bright_cyan());
    println!("{}", "  Optimization Analysis Report".bright_cyan().bold());
    println!("{}", "═".repeat(50).bright_cyan());
    println!();

    if optimizations.is_empty() {
        println!("{}", "✅ No optimization opportunities found!".bright_green());
        println!("   Your code looks well-optimized.");
        return;
    }

    // Group by category
    let mut by_category: std::collections::HashMap<String, Vec<&Optimization>> = std::collections::HashMap::new();
    for opt in optimizations {
        by_category
            .entry(opt.category.to_string())
            .or_insert_with(Vec::new)
            .push(opt);
    }

    println!("{} {} opportunities found", "📊".bright_blue(), optimizations.len());
    println!();

    for (category, opts) in by_category.iter() {
        println!("{}", format!("{}:", category).bright_yellow().bold());
        
        for opt in opts {
            let priority_str = match opt.priority {
                Priority::High => "HIGH".bright_red(),
                Priority::Medium => "MEDIUM".bright_yellow(),
                Priority::Low => "LOW".bright_blue(),
            };

            println!("   {} [{}:{}]", priority_str, opt.file.bright_cyan(), opt.line);
            println!("      {}", opt.description);
            println!("      {} {}", "💡".bright_yellow(), opt.suggestion.italic());
            println!();
        }
    }

    // Summary
    let high = optimizations.iter().filter(|o| o.priority == Priority::High).count();
    let medium = optimizations.iter().filter(|o| o.priority == Priority::Medium).count();
    let low = optimizations.iter().filter(|o| o.priority == Priority::Low).count();

    println!("{}", "═".repeat(50).bright_cyan());
    println!("{}", "Priority Summary:".bright_blue().bold());
    println!("   {} {} high priority", "🔴".bright_red(), high);
    println!("   {} {} medium priority", "🟡".bright_yellow(), medium);
    println!("   {} {} low priority", "🔵".bright_blue(), low);
    println!();
    println!("{}", "💡 Tip: Focus on high-priority items for maximum impact".italic());
}
