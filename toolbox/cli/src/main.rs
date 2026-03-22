//! Solana Toolbox CLI
//!
//! Unified command-line interface for AI-assisted Solana development
//!
//! ## Commands
//!
//! - `new instruction <name>` - Scaffold a new instruction
//! - `test generate <name>` - Generate a test template
//! - `validate all` - Check architecture compliance
//! - `deploy local` - Deploy to local validator
//! - `story implement <id>` - Implement a story
//! - `optimize analyze` - Find optimization opportunities

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use solana_toolbox::commands;

#[derive(Parser)]
#[command(name = "solana-toolbox")]
#[command(about = "AI-assisted Solana development toolbox", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new instruction with scaffolding
    New {
        #[command(subcommand)]
        resource: NewResource,
    },
    /// Test-related commands
    Test {
        #[command(subcommand)]
        action: TestAction,
    },
    /// Validate architecture compliance
    Validate {
        #[command(subcommand)]
        target: ValidateTarget,
    },
    /// Deploy the program
    Deploy {
        #[command(subcommand)]
        environment: DeployEnvironment,
    },
    /// Story implementation automation
    Story {
        #[command(subcommand)]
        action: StoryAction,
    },
    /// Optimization tools
    Optimize {
        #[command(subcommand)]
        action: OptimizeAction,
    },
    /// Run CI/CD pipeline
    Pipeline {
        /// Run cargo check (type-check)
        #[arg(long)]
        check: bool,
        /// Run cargo build-sbf (compile program)
        #[arg(long)]
        build: bool,
        /// Run cargo test (all tests)
        #[arg(long)]
        test: bool,
        /// Deploy to local validator
        #[arg(long)]
        deploy: bool,
        /// Run example client
        #[arg(long)]
        client: bool,
        /// Run all steps (check + build + test + deploy + client)
        #[arg(long)]
        all: bool,
        /// Validator startup timeout in seconds (default: 30)
        #[arg(long, default_value = "30")]
        validator_timeout: u64,
    },
}

#[derive(Subcommand)]
enum NewResource {
    /// Create a new instruction
    Instruction {
        /// Instruction name in PascalCase
        name: String,
        /// Optional data fields (format: field:type)
        #[arg(short, long, value_name = "FIELD:TYPE")]
        fields: Vec<String>,
    },
}

#[derive(Subcommand)]
enum TestAction {
    /// Generate a test from template
    Generate {
        /// Test name
        name: String,
        /// Instruction being tested
        #[arg(short, long)]
        instruction: String,
    },
}

#[derive(Subcommand)]
enum ValidateTarget {
    /// Validate all architecture rules
    All,
}

#[derive(Subcommand)]
enum DeployEnvironment {
    /// Deploy to local validator
    Local,
}

#[derive(Subcommand)]
enum StoryAction {
    /// Implement a story from stories/
    Implement {
        /// Story ID (e.g., "001", "002")
        id: String,
    },
}

#[derive(Subcommand)]
enum OptimizeAction {
    /// Analyze optimization opportunities
    Analyze,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("\n");
    println!("{}", "╔═══════════════════════════════════════╗".bright_cyan());
    println!("{}", "║   Solana Toolbox v0.1.0               ║".bright_cyan());
    println!("{}", "╚═══════════════════════════════════════╝".bright_cyan());
    println!();

    match cli.command {
        Commands::New { resource } => match resource {
            NewResource::Instruction { name, fields } => {
                commands::new_instruction(&name, &fields)?;
            }
        },
        Commands::Test { action } => match action {
            TestAction::Generate { name, instruction } => {
                commands::generate_test(&name, &instruction)?;
            }
        },
        Commands::Validate { target } => match target {
            ValidateTarget::All => {
                commands::validate_all()?;
            }
        },
        Commands::Deploy { environment } => match environment {
            DeployEnvironment::Local => {
                commands::deploy_local()?;
            }
        },
        Commands::Story { action } => match action {
            StoryAction::Implement { id } => {
                commands::implement_story(&id)?;
            }
        },
        Commands::Optimize { action } => match action {
            OptimizeAction::Analyze => {
                commands::analyze_optimizations()?;
            }
        },
        Commands::Pipeline {
            check,
            build,
            test,
            deploy,
            client,
            all,
            validator_timeout,
        } => {
            if all {
                // Run all steps
                commands::run_ci_pipeline(true, true, true, true, true, validator_timeout)?;
            } else {
                // Run selected steps
                commands::run_ci_pipeline(check, build, test, deploy, client, validator_timeout)?;
            }
        }
    }

    println!();
    Ok(())
}
