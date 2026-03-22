//! Command implementations for the toolbox CLI
//!
//! This module implements the core CLI commands for code generation, validation,
//! and architecture compliance.
//!
//! # Available Commands
//!
//! - [`new_instruction`]: Scaffold a new instruction with all layers
//! - [`generate_test`]: Generate a test template
//! - [`validate_all`]: Run architecture validation
//!
//! # Examples
//!
//! ```no_run
//! use solana_toolbox::commands::{new_instruction, validate_all};
//!
//! // Create new instruction
//! let fields = vec!["amount:u64".to_string(), "recipient:Pubkey".to_string()];
//! new_instruction("Transfer", &fields).unwrap();
//!
//! // Validate architecture
//! validate_all().unwrap();
//! ```

use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::template::{self, TemplateEngine};
use crate::validation;

/// Create a new instruction with scaffolding
///
/// Generates 4 files for a complete instruction implementation:
/// 1. Instruction enum variant (`*_instruction.rs`)
/// 2. Processor handler (`*_processor.rs`)
/// 3. Client builder (`*_client.rs`)
/// 4. LiteSVM test (`*_test.rs`)
///
/// # Arguments
///
/// * `name` - Instruction name in PascalCase (e.g., "Transfer")
/// * `fields` - Data fields in "name:type" format (e.g., ["amount:u64"])
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::commands::new_instruction;
///
/// // Simple instruction
/// new_instruction("UpdateState", &[]).unwrap();
///
/// // Instruction with data
/// let fields = vec![
///     "amount:u64".to_string(),
///     "recipient:Pubkey".to_string(),
/// ];
/// new_instruction("Transfer", &fields).unwrap();
/// ```
pub fn new_instruction(name: &str, fields: &[String]) -> Result<()> {
    println!("{}", "🚀 Creating new instruction...".bright_blue().bold());
    println!("   Name: {}", name.bright_yellow());
    
    if !fields.is_empty() {
        println!("   Fields: {}", fields.join(", "));
    }

    let engine = TemplateEngine::new()?;
    let output_dir = Path::new("toolbox/generated");
    template::ensure_output_dir(output_dir)?;

    let snake_name = template::pascal_to_snake(name);
    let data_fields = template::format_fields(fields);

    // Generate instruction variant
    let mut subs = HashMap::new();
    subs.insert("INSTRUCTION_NAME".to_string(), name.to_string());
    subs.insert(
        "INSTRUCTION_COMMENT".to_string(),
        format!("TODO: Add description"),
    );
    subs.insert("DATA_FIELDS".to_string(), data_fields.clone());

    let instruction_code = engine.load_and_render("instruction.rs.template", &subs)?;
    let instruction_file = output_dir.join(format!("{}_instruction.rs", snake_name));
    fs::write(&instruction_file, instruction_code)
        .context("Failed to write instruction file")?;
    println!("   {} {}", "✅".green(), instruction_file.display());

    // Generate processor handler
    subs.insert("process_function".to_string(), format!("process_{}", snake_name));
    subs.insert(
        "PROCESS_DESCRIPTION".to_string(),
        format!("Process {} instruction", name),
    );

    let processor_code = engine.load_and_render("processor_handler.rs.template", &subs)?;
    let processor_file = output_dir.join(format!("{}_processor.rs", snake_name));
    fs::write(&processor_file, processor_code)
        .context("Failed to write processor file")?;
    println!("   {} {}", "✅".green(), processor_file.display());

    // Generate client builder
    subs.insert("builder_function".to_string(), snake_name.clone());
    subs.insert(
        "BUILDER_DESCRIPTION".to_string(),
        format!("Build {} instruction", name),
    );

    let client_code = engine.load_and_render("client_builder.rs.template", &subs)?;
    let client_file = output_dir.join(format!("{}_client.rs", snake_name));
    fs::write(&client_file, client_code)
        .context("Failed to write client file")?;
    println!("   {} {}", "✅".green(), client_file.display());

    // Generate test
    subs.insert("test_name".to_string(), format!("test_{}", snake_name));
    subs.insert(
        "TEST_DESCRIPTION".to_string(),
        format!("Test {} instruction", name),
    );
    subs.insert("SETUP_CODE".to_string(), "let authority = Keypair::new();".to_string());
    subs.insert(
        "INSTRUCTION_BUILD".to_string(),
        format!("// let instruction = instructions::{}(...);", snake_name),
    );
    subs.insert("ASSERTIONS".to_string(), "// TODO: Add assertions".to_string());

    let test_code = engine.load_and_render("test.rs.template", &subs)?;
    let test_file = output_dir.join(format!("{}_test.rs", snake_name));
    fs::write(&test_file, test_code)
        .context("Failed to write test file")?;
    println!("   {} {}", "✅".green(), test_file.display());

    println!();
    println!("{}", "📝 Next steps:".bright_cyan());
    println!("   1. Review generated files in {}/", output_dir.display());
    println!("   2. Add instruction variant to src/instructions.rs");
    println!("   3. Add processor handler to src/processor.rs");
    println!("   4. Add client builder to client/instructions.rs");
    println!("   5. Add test to src/lib.rs #[cfg(test)] mod tests");
    println!("   6. Run: cargo test test_{}", snake_name);

    Ok(())
}

/// Generate a test from template
///
/// Creates a LiteSVM test template for a specific instruction.
///
/// # Arguments
///
/// * `test_name` - Test function name (e.g., "test_transfer")
/// * `instruction` - Instruction being tested (e.g., "Transfer")
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::commands::generate_test;
///
/// generate_test("test_transfer", "Transfer").unwrap();
/// ```
pub fn generate_test(test_name: &str, instruction: &str) -> Result<()> {
    println!("{}", "🧪 Generating test...".bright_blue().bold());
    println!("   Test: {}", test_name.bright_yellow());
    println!("   Instruction: {}", instruction.bright_yellow());

    let engine = TemplateEngine::new()?;
    let output_dir = Path::new("toolbox/generated");
    template::ensure_output_dir(output_dir)?;

    let mut subs = HashMap::new();
    subs.insert("test_name".to_string(), test_name.to_string());
    subs.insert(
        "TEST_DESCRIPTION".to_string(),
        format!("Test {} instruction", instruction),
    );
    subs.insert("INSTRUCTION_NAME".to_string(), instruction.to_string());
    subs.insert("SETUP_CODE".to_string(), "let authority = Keypair::new();".to_string());
    subs.insert(
        "INSTRUCTION_BUILD".to_string(),
        "// let instruction = instructions::...(...);".to_string(),
    );
    subs.insert("ASSERTIONS".to_string(), "// TODO: Add assertions".to_string());

    let test_code = engine.load_and_render("test.rs.template", &subs)?;
    let test_file = output_dir.join(format!("{}.rs", test_name));
    fs::write(&test_file, test_code)
        .context("Failed to write test file")?;

    println!("   {} {}", "✅".green(), test_file.display());
    println!();
    println!("{}", "📝 Next steps:".bright_cyan());
    println!("   1. Review {}", test_file.display());
    println!("   2. Fill in setup code and assertions");
    println!("   3. Add to src/lib.rs #[cfg(test)] mod tests");
    println!("   4. Run: cargo test {}", test_name);

    Ok(())
}

/// Validate architecture compliance
///
/// Runs all architecture validation checks against AGENTS.md guidelines:
/// - Required files exist
/// - Proper file organization
/// - Separation of concerns
/// - Naming conventions
///
/// # Errors
///
/// Returns an error if validation fails (errors found).
///
/// # Examples
///
/// ```no_run
/// use insurance_toolbox::commands::validate_all;
///
/// if let Err(e) = validate_all() {
///     eprintln!("Validation failed: {}", e);
/// }
/// ```
pub fn validate_all() -> Result<()> {
    println!("{}", "🔍 Validating architecture...".bright_blue().bold());
    println!();

    let report = validation::validate_architecture()?;
    report.print();

    if report.has_errors() {
        anyhow::bail!("Architecture validation failed");
    }

    Ok(())
}

/// Deploy to local validator
pub fn deploy_local() -> Result<()> {
    use crate::deployment;

    println!("{}", "🚀 Deploying to local validator...".bright_blue().bold());
    println!();

    deployment::run_deployment_pipeline()?;

    Ok(())
}

/// Implement a story
pub fn implement_story(story_id: &str) -> Result<()> {
    use crate::story;

    println!("{}", "📖 Implementing story...".bright_blue().bold());
    println!();

    // Parse the story
    let story = story::parse_story(story_id)
        .with_context(|| format!("Failed to parse story {}", story_id))?;

    // Print story summary
    story::print_story_summary(&story);
    println!();

    // Check if story is already complete
    if story.status.contains("DONE") || story.status.contains("✅") {
        println!("{}", "ℹ️  This story is already marked as DONE".bright_yellow());
        println!("   Review the implementation in the codebase.");
        return Ok(());
    }

    // Extract instructions mentioned in the story
    let instructions = story::extract_instructions_from_story(&story);
    
    if !instructions.is_empty() {
        println!("{}", "🔍 Detected Instructions:".bright_blue());
        for instruction in &instructions {
            println!("   • {}", instruction.bright_yellow());
        }
        println!();
    }

    // Provide implementation guidance
    println!("{}", "📝 Implementation Steps:".bright_cyan().bold());
    println!();

    if !instructions.is_empty() {
        println!("1. Generate instruction scaffolds:");
        for instruction in &instructions {
            println!("   {}", format!("toolbox new instruction {}", instruction).bright_white());
        }
        println!();
    }

    if !story.tasks.is_empty() {
        let incomplete_tasks: Vec<_> = story.tasks.iter().filter(|t| !t.completed).collect();
        
        if !incomplete_tasks.is_empty() {
            println!("2. Complete remaining tasks:");
            for task in incomplete_tasks {
                println!("   ⬜ {} {}", task.id.bright_yellow(), task.description);
            }
            println!();
        }
    }

    println!("3. Run validation:");
    println!("   {}", "toolbox validate all".bright_white());
    println!();

    println!("4. Run tests:");
    println!("   {}", "cargo test".bright_white());
    println!();

    println!("{}", "💡 Tip: Use the generated code as starting points, then fill in business logic".italic());
    
    Ok(())
}

/// Analyze optimization opportunities
pub fn analyze_optimizations() -> Result<()> {
    use crate::optimization;

    println!("{}", "🔬 Analyzing optimizations...".bright_blue().bold());
    println!();

    let optimizations = optimization::analyze_codebase()
        .context("Failed to analyze codebase")?;

    optimization::print_optimization_report(&optimizations);

    Ok(())
}

/// Run the CI/CD pipeline with configurable steps
///
/// Executes a configurable pipeline with step selection:
/// - **check**: Run `cargo check` for type-checking
/// - **build**: Run `cargo build-sbf` to compile program
/// - **test**: Run `cargo test` for all tests
/// - **deploy**: Deploy to local validator
/// - **client**: Run example client
///
/// # Arguments
///
/// * `check` - Enable cargo check step
/// * `build` - Enable cargo build-sbf step
/// * `test` - Enable cargo test step
/// * `deploy` - Enable deployment step
/// * `client` - Enable client execution step
/// * `validator_timeout` - Seconds to wait for validator startup
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::commands::run_ci_pipeline;
///
/// // Run build + test
/// run_ci_pipeline(false, true, true, false, false, 30).unwrap();
///
/// // Run all steps
/// run_ci_pipeline(true, true, true, true, true, 30).unwrap();
/// ```
pub fn run_ci_pipeline(
    check: bool,
    build: bool,
    test: bool,
    deploy: bool,
    client: bool,
    validator_timeout: u64,
) -> Result<()> {
    use crate::deployment::{PipelineConfig, run_pipeline};

    let config = PipelineConfig {
        do_check: check,
        do_build: build,
        do_test: test,
        do_deploy: deploy,
        do_client: client,
        validator_timeout,
    };

    // If no steps selected, default to build + test
    if !config.has_any_step() {
        let default_config = PipelineConfig::default();
        run_pipeline(default_config)?;
    } else {
        run_pipeline(config)?;
    }

    Ok(())
}
