//! Deployment utilities for Solana programs
//!
//! This module provides functions for building, deploying, and managing Solana programs
//! on local test validators.
//!
//! # Examples
//!
//! ```no_run
//! use solana_toolbox::deployment::run_deployment_pipeline;
//!
//! // Run full deployment pipeline: build → start validator → deploy
//! run_deployment_pipeline().unwrap();
//! ```
//!
//! # Pipeline Steps
//!
//! 1. Check prerequisites (solana CLI, cargo build-sbf)
//! 2. Build program to BPF with `cargo build-sbf`
//! 3. Start local test validator
//! 4. Deploy program and extract program ID
//! 5. Display cluster information

use anyhow::{Context, Result};
use colored::Colorize;
use std::env;
use std::fs;
use std::io::Read;
use std::process::{Command, Stdio};
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_LEDGER_DIR: &str = "target/test-ledger";

const DEFAULT_LOCAL_RPC_URL: &str = "http://127.0.0.1:8899";
const DEFAULT_PROGRAM_SO: &str = "target/deploy/insurance.so";

fn validator_ledger_dir() -> String {
    if let Ok(custom_dir) = env::var("SOLANA_TOOLBOX_LEDGER_DIR") {
        return custom_dir;
    }

    // Windows frequently blocks validator ledger creation in Desktop-synced paths.
    if cfg!(target_os = "windows") {
        return make_fallback_ledger_dir();
    }

    DEFAULT_LEDGER_DIR.to_string()
}

fn make_fallback_ledger_dir() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    env::temp_dir()
        .join(format!("solana-toolbox-ledger-{ts}"))
        .to_string_lossy()
        .to_string()
}

/// Configuration for local build/deploy/client workflow.
pub struct LocalRunConfig {
    pub airdrop_sol: u64,
    pub keep_validator: bool,
    pub skip_build: bool,
    pub skip_client: bool,
    pub validator_timeout: u64,
    pub rpc_url: String,
    pub program_so: String,
}

impl Default for LocalRunConfig {
    fn default() -> Self {
        Self {
            airdrop_sol: 2,
            keep_validator: false,
            skip_build: false,
            skip_client: false,
            validator_timeout: 30,
            rpc_url: DEFAULT_LOCAL_RPC_URL.to_string(),
            program_so: DEFAULT_PROGRAM_SO.to_string(),
        }
    }
}

struct ValidatorGuard {
    child: Option<std::process::Child>,
    keep_validator: bool,
}

impl ValidatorGuard {
    fn new(keep_validator: bool) -> Self {
        Self {
            child: None,
            keep_validator,
        }
    }

    fn set_child(&mut self, child: Option<std::process::Child>) {
        self.child = child;
    }
}

impl Drop for ValidatorGuard {
    fn drop(&mut self) {
        if self.keep_validator {
            return;
        }

        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Check if solana-test-validator is available
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::deployment::check_solana_cli;
///
/// if check_solana_cli().unwrap() {
///     println!("Solana CLI is installed");
/// }
/// ```
pub fn check_solana_cli() -> Result<bool> {
    let output = Command::new("solana")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    Ok(output.is_ok() && output.unwrap().success())
}

/// Check if cargo build-sbf is available
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::deployment::check_cargo_build_sbf;
///
/// if !check_cargo_build_sbf().unwrap() {
///     eprintln!("Please install Solana CLI tools");
/// }
/// ```
pub fn check_cargo_build_sbf() -> Result<bool> {
    let output = Command::new("cargo")
        .arg("build-sbf")
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    Ok(output.is_ok() && output.unwrap().success())
}

/// Run cargo check for type checking
///
/// Executes `cargo check` to verify types without building a binary.
///
/// # Errors
///
/// Returns an error if type-checking fails.
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::deployment::check_types;
///
/// check_types().unwrap();
/// println!("Type check passed");
/// ```
pub fn check_types() -> Result<()> {
    println!("   {} Running cargo check...", "🔍".bright_yellow());

    let status = Command::new("cargo")
        .arg("check")
        .status()
        .context("Failed to run cargo check")?;

    if !status.success() {
        anyhow::bail!("cargo check failed");
    }

    println!("   {} Type check passed", "✅".green());
    Ok(())
}

/// Build the Solana program to BPF
///
/// Executes `cargo build-sbf` to compile the program to BPF bytecode.
///
/// # Errors
///
/// Returns an error if the build fails or cargo build-sbf is not available.
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::deployment::build_program;
///
/// build_program().unwrap();
/// println!("Program built successfully");
/// ```
pub fn build_program() -> Result<()> {
    println!("   {} Building Solana program...", "🔨".bright_yellow());

    let status = Command::new("cargo")
        .arg("build-sbf")
        .status()
        .context("Failed to run cargo build-sbf")?;

    if !status.success() {
        anyhow::bail!("cargo build-sbf failed");
    }

    println!("   {} Build successful", "✅".green());
    Ok(())
}

/// Run cargo test
///
/// Executes `cargo test` to run all tests (including LiteSVM tests).
///
/// # Errors
///
/// Returns an error if any test fails.
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::deployment::run_tests;
///
/// run_tests().unwrap();
/// println!("All tests passed");
/// ```
pub fn run_tests() -> Result<()> {
    println!("   {} Running cargo test...", "🧪".bright_yellow());

    let output = Command::new("cargo")
        .arg("test")
        .arg("--")
        .arg("--nocapture")
        .output()
        .context("Failed to run cargo test")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Print output
    print!("{}", stdout);
    print!("{}", stderr);

    if !output.status.success() {
        anyhow::bail!("cargo test failed");
    }

    // Extract test counts
    let passed = stdout.lines()
        .filter(|line| line.contains("passed"))
        .find_map(|line| {
            line.split_whitespace()
                .find_map(|word| word.parse::<usize>().ok())
        })
        .unwrap_or(0);

    println!("   {} Tests passed: {}", "✅".green(), passed);
    Ok(())
}

/// Ensure validator is running, auto-starting if needed
///
/// Checks if a validator is running at localhost:8899. If not, starts
/// solana-test-validator in the background and waits for it to be ready.
///
/// # Arguments
///
/// * `timeout_secs` - Maximum seconds to wait for validator to become ready (default: 30)
///
/// # Errors
///
/// Returns an error if the validator fails to start or doesn't become ready within timeout.
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::deployment::ensure_validator;
///
/// // Wait up to 30 seconds for validator
/// ensure_validator(30).unwrap();
/// ```
pub fn ensure_validator(timeout_secs: u64) -> Result<()> {
    let _ = ensure_validator_with_options(timeout_secs, "http://localhost:8899", 2)?;
    Ok(())
}

/// Ensure validator is running, auto-starting if needed.
///
/// Returns a child handle when this function started the validator process.
pub fn ensure_validator_with_options(
    timeout_secs: u64,
    rpc_url: &str,
    airdrop_sol: u64,
) -> Result<Option<std::process::Child>> {
    println!("   {} Checking validator status...", "🔍".bright_yellow());

    // Check if validator is already running
    let check = Command::new("solana")
        .args(["cluster-version", "-u", rpc_url])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if check.is_ok() && check.unwrap().success() {
        println!("   {} Validator already running at {}", "✅".green(), rpc_url);
        return Ok(None);
    }

    println!("   {} Validator not detected — starting solana-test-validator...", "⚠️".yellow());

    let configured_ledger = validator_ledger_dir();
    let log_hint = "target/validator.log";
    let _ = fs::create_dir_all("target");

    // Use a fallback temp ledger if the configured path appears stale or locked.
    let ledger_to_use = if Path::new(&configured_ledger).exists() {
        let fallback = make_fallback_ledger_dir();
        println!(
            "   {} Existing ledger detected at {} — using fresh ledger {}",
            "ℹ️".bright_blue(),
            configured_ledger,
            fallback
        );
        fallback
    } else {
        configured_ledger
    };

    // Start validator in background with stdout+stderr captured for crash diagnostics.
    let mut child = Command::new("solana-test-validator")
        .arg("--reset")
        .arg("--ledger")
        .arg(&ledger_to_use)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to start solana-test-validator with ledger {ledger_to_use}"))?;

    let pid = child.id();
    println!("   {} Validator started (PID: {})", "✅".green(), pid);
    
    // Give it a moment to fail if there's a startup error
    thread::sleep(Duration::from_millis(1000));
    
    // Check if the process is still alive
    match child.try_wait() {
        Ok(Some(status)) => {
            // Process already exited - check stderr for error
            let mut stdout_output = String::new();
            let mut stderr_output = String::new();
            if let Some(mut stdout) = child.stdout.take() {
                let _ = stdout.read_to_string(&mut stdout_output);
            }
            if let Some(mut stderr) = child.stderr.take() {
                let _ = stderr.read_to_string(&mut stderr_output);
            }
            let combined_output = format!("{}\n{}", stdout_output, stderr_output);
            let _ = fs::write(log_hint, &combined_output);
            
            if cfg!(target_os = "windows")
                && (combined_output.contains("1314")
                    || combined_output.contains("privilege")
                    || combined_output.contains("Access is denied")
                    || combined_output.contains("os error 5"))
            {
                eprintln!("\n   {} {}", "❌".red(), "Validator failed to start due to Windows permissions.".red());
                eprintln!("   {} On Windows, solana-test-validator requires one of:", "ℹ️".bright_blue());
                eprintln!("      1. {} enabled (Settings → For developers)", "Developer Mode".bright_white());
                eprintln!("      2. Running PowerShell/VS Code as {}", "Administrator".bright_white());
                eprintln!("\n   {} This often appears as \"Access is denied (os error 5)\" during ledger creation.", "ℹ️".bright_blue());
                eprintln!("   {} Enable Developer Mode and restart to fix permanently.\n", "💡".bright_yellow());
                anyhow::bail!("Validator requires admin privileges or Developer Mode on Windows");
            } else if combined_output.contains("Access is denied")
                || combined_output.contains("os error 5")
                || combined_output.contains("Failed to create ledger")
            {
                eprintln!("\n   {} Validator failed to access ledger path.", "❌".red());
                eprintln!(
                    "   {} Ledger used: {}",
                    "ℹ️".bright_blue(),
                    ledger_to_use
                );
                eprintln!(
                    "   {} Crash output saved to {}",
                    "ℹ️".bright_blue(),
                    log_hint
                );
                anyhow::bail!("Validator failed to access ledger directory");
            } else {
                eprintln!("\n   {} Validator crashed on startup:", "❌".red());
                eprintln!("{}", combined_output);
                eprintln!("   {} Crash output saved to {}", "ℹ️".bright_blue(), log_hint);
                anyhow::bail!("Validator exited with status: {}", status);
            }
        }
        Ok(None) => {
            // Process is still running - good!
            println!("   {} Waiting for validator to be ready (timeout: {}s)...", "⏳".bright_yellow(), timeout_secs);
        }
        Err(e) => {
            anyhow::bail!("Failed to check validator status: {}", e);
        }
    }

    // Wait for validator to be ready
    let iterations = timeout_secs * 2; // Check every 500ms
    for i in 1..=iterations {
        thread::sleep(Duration::from_millis(500));
        
        let check = Command::new("solana")
            .args(["cluster-version", "-u", rpc_url])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if check.is_ok() && check.unwrap().success() {
            let elapsed_ms = i * 500;
            println!("   {} Validator ready after {}.{}s", "✅".green(), elapsed_ms / 1000, (elapsed_ms % 1000) / 100);
            
            // Fund the default keypair
            println!("   {} Funding default keypair...", "💰".bright_yellow());
            let airdrop = Command::new("solana")
                .args(["airdrop", &airdrop_sol.to_string(), "-u", rpc_url])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            
            if airdrop.is_ok() && airdrop.unwrap().success() {
                println!("   {} Airdrop successful ({} SOL)", "✅".green(), airdrop_sol);
            } else {
                println!("   {} Airdrop failed (keypair may already be funded)", "⚠️".yellow());
            }
            
            return Ok(Some(child));
        }
    }

    anyhow::bail!("Validator did not become ready within {}s", timeout_secs);
}

/// Start solana-test-validator in the background
///
/// Starts a local test validator and waits for it to become ready.
/// If a validator is already running, this function succeeds immediately.
///
/// # Errors
///
/// Returns an error if the validator fails to start or doesn't become ready within 5 seconds.
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::deployment::start_validator;
///
/// start_validator().unwrap();
/// println!("Validator is ready");
/// ```
pub fn start_validator() -> Result<()> {
    ensure_validator(5)
}

/// Deploy the program to the local validator
///
/// Deploys the compiled program and extracts the program ID from output.
///
/// # Returns
///
/// The deployed program's public key as a string.
///
/// # Errors
///
/// Returns an error if:
/// - The program binary doesn't exist at `target/deploy/*.so`
/// - The deployment command fails
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::deployment::deploy_program;
///
/// let program_id = deploy_program().unwrap();
/// println!("Deployed program ID: {}", program_id);
/// ```
pub fn deploy_program() -> Result<String> {
    deploy_program_with_options("http://localhost:8899", "target/deploy/insurance.so")
}

/// Deploy program with explicit RPC URL and binary path.
pub fn deploy_program_with_options(rpc_url: &str, program_so: &str) -> Result<String> {
    println!("   {} Deploying program...", "📦".bright_yellow());

    if !Path::new(program_so).exists() {
        anyhow::bail!("Program binary not found: {}", program_so);
    }

    let output = Command::new("solana")
        .args(["program", "deploy", "-u", rpc_url, program_so])
        .output()
        .context("Failed to deploy program")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Deploy failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Extract program ID from output (format: "Program Id: <pubkey>")
    let program_id = stdout
        .lines()
        .find(|line| line.contains("Program Id:"))
        .and_then(|line| line.split_whitespace().last())
        .unwrap_or("unknown")
        .to_string();

    println!("   {} Program deployed", "✅".green());
    println!("   {} Program ID: {}", "📍".bright_cyan(), program_id.bright_yellow());

    Ok(program_id)
}

/// Build program with cargo build-sbf and fallback to solana program build.
pub fn build_program_with_fallback() -> Result<()> {
    println!("   {} Building Solana program...", "🔨".bright_yellow());

    let cargo_status = Command::new("cargo").arg("build-sbf").status();
    if cargo_status.is_ok() && cargo_status.unwrap().success() {
        println!("   {} Build successful (cargo build-sbf)", "✅".green());
        return Ok(());
    }

    println!(
        "   {} cargo build-sbf unavailable/failed; trying solana program build...",
        "⚠️".yellow()
    );

    let solana_status = Command::new("solana")
        .args(["program", "build"])
        .status()
        .context("Failed to run solana program build")?;

    if !solana_status.success() {
        anyhow::bail!("solana program build failed");
    }

    println!("   {} Build successful (solana program build)", "✅".green());
    Ok(())
}

fn ensure_default_keypair() -> Result<()> {
    let output = Command::new("solana")
        .args(["config", "get"])
        .output()
        .context("Failed to read solana config")?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut keypair_path = stdout
        .lines()
        .find(|line| line.contains("Keypair Path:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if keypair_path.is_empty() || keypair_path == "ASK" || keypair_path.starts_with("prompt:") {
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        keypair_path = format!("{home}/.config/solana/id.json");

        let status = Command::new("solana")
            .args(["config", "set", "--keypair", &keypair_path])
            .status()
            .context("Failed to set default keypair path")?;

        if !status.success() {
            anyhow::bail!("Could not set keypair path in solana config");
        }
    }

    if Path::new(&keypair_path).exists() {
        return Ok(());
    }

    if let Some(parent) = Path::new(&keypair_path).parent() {
        fs::create_dir_all(parent).context("Failed to create keypair directory")?;
    }

    let status = Command::new("solana-keygen")
        .args(["new", "--no-bip39-passphrase", "-o", &keypair_path, "-f"])
        .status()
        .context("Failed to generate keypair")?;

    if !status.success() {
        anyhow::bail!("solana-keygen failed creating keypair");
    }

    Ok(())
}

/// Script-equivalent local workflow: start validator, build, deploy, and optionally run client.
pub fn run_local_workflow(config: LocalRunConfig) -> Result<()> {
    println!("{}", "🚀 Running local workflow...".bright_blue().bold());

    if !check_solana_cli()? {
        anyhow::bail!("solana CLI not found in PATH");
    }

    let validator_check = Command::new("solana-test-validator")
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if validator_check.is_err() || !validator_check.unwrap().success() {
        anyhow::bail!("solana-test-validator not found in PATH");
    }

    ensure_default_keypair()?;

    let mut guard = ValidatorGuard::new(config.keep_validator);
    let started = ensure_validator_with_options(
        config.validator_timeout,
        &config.rpc_url,
        config.airdrop_sol,
    )?;
    guard.set_child(started);

    let set_url = Command::new("solana")
        .args(["config", "set", "--url", &config.rpc_url])
        .status()
        .context("Failed to set solana config url")?;
    if !set_url.success() {
        anyhow::bail!("Could not set solana config URL to {}", config.rpc_url);
    }

    if !config.skip_build {
        build_program_with_fallback()?;
    } else {
        println!("   {} Skipping build (per flag)", "⊝".bright_black());
    }

    if !Path::new(&config.program_so).exists() {
        anyhow::bail!("Program artifact not found: {}", config.program_so);
    }

    let _program_id = deploy_program_with_options(&config.rpc_url, &config.program_so)?;

    if !config.skip_client {
        run_client()?;
    } else {
        println!("   {} Skipping client run (per flag)", "⊝".bright_black());
    }

    if config.keep_validator {
        println!("   {} Validator left running", "ℹ️".bright_blue());
    }

    Ok(())
}

/// Run the example client
///
/// Executes the example client binary or uses `cargo run --bin client`.
///
/// # Errors
///
/// Returns an error if the client fails to run.
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::deployment::run_client;
///
/// run_client().unwrap();
/// ```
pub fn run_client() -> Result<()> {
    println!("   {} Running example client...", "🚀".bright_yellow());

    // Try cargo run --bin client first, fallback to --example client
    let status = Command::new("cargo")
        .args(&["run", "--bin", "client"])
        .status();

    let result = if status.is_ok() && status.unwrap().success() {
        Ok(())
    } else {
        // Try example client as fallback
        let example_status = Command::new("cargo")
            .args(&["run", "--example", "client"])
            .status()
            .context("Failed to run example client")?;

        if !example_status.success() {
            anyhow::bail!("Example client failed");
        }
        Ok(())
    };

    if result.is_ok() {
        println!("   {} Client completed successfully", "✅".green());
    }

    result
}

/// Get the current cluster endpoint
pub fn get_cluster_info() -> Result<String> {
    let output = Command::new("solana")
        .args(&["config", "get"])
        .output()
        .context("Failed to get cluster config")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let rpc_url = stdout
        .lines()
        .find(|line| line.contains("RPC URL:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim())
        .unwrap_or("unknown")
        .to_string();

    Ok(rpc_url)
}

/// Pipeline step result
#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    Pass,
    Fail,
    Skip,
}

impl StepResult {
    fn icon(&self) -> &'static str {
        match self {
            StepResult::Pass => "✅",
            StepResult::Fail => "❌",
            StepResult::Skip => "⊝",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            StepResult::Pass => "PASS",
            StepResult::Fail => "FAIL",
            StepResult::Skip => "SKIP",
        }
    }
}

/// Pipeline configuration
pub struct PipelineConfig {
    pub do_check: bool,
    pub do_build: bool,
    pub do_test: bool,
    pub do_deploy: bool,
    pub do_client: bool,
    pub validator_timeout: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            do_check: false,
            do_build: true,
            do_test: true,
            do_deploy: false,
            do_client: false,
            validator_timeout: 30,
        }
    }
}

impl PipelineConfig {
    /// Create config with all steps enabled
    pub fn all() -> Self {
        Self {
            do_check: true,
            do_build: true,
            do_test: true,
            do_deploy: true,
            do_client: true,
            validator_timeout: 30,
        }
    }

    /// Check if any step is enabled
    pub fn has_any_step(&self) -> bool {
        self.do_check || self.do_build || self.do_test || self.do_deploy || self.do_client
    }
}

/// Run the CI/CD pipeline with configurable steps
///
/// Executes a configurable pipeline with the following steps:
/// 1. **cargo check** - Type-check without building (if `do_check`)
/// 2. **cargo build-sbf** - Build Solana program to BPF (if `do_build`)
/// 3. **cargo test** - Run all tests including LiteSVM (if `do_test`)
/// 4. **solana program deploy** - Deploy to local validator (if `do_deploy`)
/// 5. **cargo run --bin client** - Run example client (if `do_client`)
///
/// The pipeline tracks results for each step and provides a summary at the end.
/// If any step fails, the pipeline stops immediately and returns an error.
///
/// # Arguments
///
/// * `config` - Pipeline configuration with step flags and timeout
///
/// # Errors
///
/// Returns an error if any enabled step fails.
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::deployment::{run_pipeline, PipelineConfig};
///
/// // Run build + test
/// let config = PipelineConfig {
///     do_build: true,
///     do_test: true,
///     ..Default::default()
/// };
/// run_pipeline(config).unwrap();
///
/// // Run all steps
/// run_pipeline(PipelineConfig::all()).unwrap();
/// ```
pub fn run_pipeline(config: PipelineConfig) -> Result<()> {
    use chrono::Utc;
    
    let start_time = Utc::now();
    
    println!("{}", "═".repeat(60).bright_cyan());
    println!("{}", "  Solana CI/CD Pipeline".bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_cyan());
    println!("   Started: {}", start_time.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("   Steps  : {}{}{}{}{}", 
        if config.do_check { "check " } else { "" },
        if config.do_build { "build " } else { "" },
        if config.do_test { "test " } else { "" },
        if config.do_deploy { "deploy " } else { "" },
        if config.do_client { "client" } else { "" },
    );
    println!();

    let mut results: Vec<(&str, StepResult)> = Vec::new();

    // Step 1: cargo check
    println!("{}", "──────────────────────────────────────────────────────────".bright_black());
    if config.do_check {
        println!("{}", "Step 1 — cargo check".bright_blue().bold());
        println!("{}", "──────────────────────────────────────────────────────────".bright_black());
        match check_types() {
            Ok(_) => {
                results.push(("check", StepResult::Pass));
                println!();
            }
            Err(e) => {
                results.push(("check", StepResult::Fail));
                println!();
                println!("{} Type-check failed: {}", "❌".red(), e);
                print_summary(&results, start_time);
                anyhow::bail!("Pipeline failed at step: check");
            }
        }
    } else {
        results.push(("check", StepResult::Skip));
    }

    // Step 2: cargo build-sbf
    if config.do_build {
        println!("{}", "──────────────────────────────────────────────────────────".bright_black());
        println!("{}", "Step 2 — cargo build-sbf".bright_blue().bold());
        println!("{}", "──────────────────────────────────────────────────────────".bright_black());
        match build_program() {
            Ok(_) => {
                results.push(("build", StepResult::Pass));
                
                // Show binary size
                let so_path = "target/deploy/insurance.so";
                if Path::new(so_path).exists() {
                    let metadata = std::fs::metadata(so_path)?;
                    let size_kb = metadata.len() / 1024;
                    println!("   {} Binary: {} ({} KB)", "📦".bright_cyan(), so_path, size_kb);
                }
                println!();
            }
            Err(e) => {
                results.push(("build", StepResult::Fail));
                println!();
                println!("{} Build failed: {}", "❌".red(), e);
                print_summary(&results, start_time);
                anyhow::bail!("Pipeline failed at step: build");
            }
        }
    } else {
        results.push(("build", StepResult::Skip));
    }

    // Step 3: cargo test
    if config.do_test {
        println!("{}", "──────────────────────────────────────────────────────────".bright_black());
        println!("{}", "Step 3 — cargo test".bright_blue().bold());
        println!("{}", "──────────────────────────────────────────────────────────".bright_black());
        
        // Verify .so exists
        let so_path = "target/deploy/insurance.so";
        if !Path::new(so_path).exists() {
            results.push(("test", StepResult::Fail));
            println!();
            println!("{} Program binary not found: {}", "❌".red(), so_path);
            println!("   Run with --build first to compile the program.");
            print_summary(&results, start_time);
            anyhow::bail!("Pipeline failed at step: test (missing binary)");
        }

        match run_tests() {
            Ok(_) => {
                results.push(("test", StepResult::Pass));
                println!();
            }
            Err(e) => {
                results.push(("test", StepResult::Fail));
                println!();
                println!("{} Tests failed: {}", "❌".red(), e);
                print_summary(&results, start_time);
                anyhow::bail!("Pipeline failed at step: test");
            }
        }
    } else {
        results.push(("test", StepResult::Skip));
    }

    // Step 4: solana program deploy
    if config.do_deploy {
        println!("{}", "──────────────────────────────────────────────────────────".bright_black());
        println!("{}", "Step 4 — solana program deploy".bright_blue().bold());
        println!("{}", "──────────────────────────────────────────────────────────".bright_black());

        // Warn if deploying without tests
        if !config.do_test {
            println!("   {} Deploying without running tests", "⚠️".yellow());
        }

        // Ensure validator is running
        match ensure_validator(config.validator_timeout) {
            Ok(_) => {}
            Err(e) => {
                results.push(("deploy", StepResult::Fail));
                println!();
                println!("{} Validator startup failed: {}", "❌".red(), e);
                print_summary(&results, start_time);
                anyhow::bail!("Pipeline failed at step: deploy (validator)");
            }
        }

        match deploy_program() {
            Ok(_program_id) => {
                results.push(("deploy", StepResult::Pass));
                println!("   {} RPC: http://localhost:8899", "🌐".bright_cyan());
                println!();
            }
            Err(e) => {
                results.push(("deploy", StepResult::Fail));
                println!();
                println!("{} Deployment failed: {}", "❌".red(), e);
                print_summary(&results, start_time);
                anyhow::bail!("Pipeline failed at step: deploy");
            }
        }
    } else {
        results.push(("deploy", StepResult::Skip));
    }

    // Step 5: cargo run --bin client
    if config.do_client {
        println!("{}", "──────────────────────────────────────────────────────────".bright_black());
        println!("{}", "Step 5 — cargo run --bin client".bright_blue().bold());
        println!("{}", "──────────────────────────────────────────────────────────".bright_black());

        // Warn if running client without fresh deploy
        if !config.do_deploy {
            println!("   {} Running client without fresh deploy", "⚠️".yellow());
        }

        // Ensure validator is running
        if !config.do_deploy {
            match ensure_validator(config.validator_timeout) {
                Ok(_) => {}
                Err(e) => {
                    results.push(("client", StepResult::Fail));
                    println!();
                    println!("{} Validator startup failed: {}", "❌".red(), e);
                    print_summary(&results, start_time);
                    anyhow::bail!("Pipeline failed at step: client (validator)");
                }
            }
        }

        match run_client() {
            Ok(_) => {
                results.push(("client", StepResult::Pass));
                println!();
            }
            Err(e) => {
                results.push(("client", StepResult::Fail));
                println!();
                println!("{} Client failed: {}", "❌".red(), e);
                print_summary(&results, start_time);
                anyhow::bail!("Pipeline failed at step: client");
            }
        }
    } else {
        results.push(("client", StepResult::Skip));
    }

    print_summary(&results, start_time);
    Ok(())
}

/// Print pipeline summary
fn print_summary(results: &[(&str, StepResult)], start_time: chrono::DateTime<chrono::Utc>) {
    use chrono::Utc;
    
    let end_time = Utc::now();
    let duration = end_time.signed_duration_since(start_time);
    
    println!("{}", "═".repeat(60).bright_cyan());
    println!("{}", "  Pipeline Summary".bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_cyan());
    println!("   Finished: {}", end_time.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("   Duration: {}.{:03}s", duration.num_seconds(), duration.num_milliseconds() % 1000);
    println!();
    
    for (step, result) in results {
        let color = match result {
            StepResult::Pass => colored::Color::Green,
            StepResult::Fail => colored::Color::Red,
            StepResult::Skip => colored::Color::BrightBlack,
        };
        
        println!("   {} {}  {}", 
            result.icon(), 
            result.label().color(color).bold(),
            step
        );
    }
    println!();

    let _passed = results.iter().filter(|(_, r)| *r == StepResult::Pass).count();
    let failed = results.iter().filter(|(_, r)| *r == StepResult::Fail).count();
    let _skipped = results.iter().filter(|(_, r)| *r == StepResult::Skip).count();

    if failed > 0 {
        println!("{}", "═".repeat(60).red());
        println!("{}", "  ❌ Pipeline Failed".red().bold());
        println!("{}", "═".repeat(60).red());
    } else {
        println!("{}", "═".repeat(60).green());
        println!("{}", "  ✅ All Steps Passed".green().bold());
        println!("{}", "═".repeat(60).green());
    }
}

/// Run the full deployment pipeline
///
/// Executes the complete deployment workflow:
/// 1. Validates prerequisites
/// 2. Builds the program
/// 3. Starts the local validator
/// 4. Deploys the program
/// 5. Displays cluster info and next steps
///
/// # Errors
///
/// Returns an error if any step fails.
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::deployment::run_deployment_pipeline;
///
/// run_deployment_pipeline().unwrap();
/// ```
pub fn run_deployment_pipeline() -> Result<()> {
    let config = PipelineConfig {
        do_check: false,
        do_build: true,
        do_test: false,
        do_deploy: true,
        do_client: false,
        validator_timeout: 30,
    };
    
    run_pipeline(config)
}
