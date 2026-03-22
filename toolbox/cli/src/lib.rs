//! Solana Toolbox CLI Library
//!
//! This library provides the core functionality for Solana development
//! toolbox, including code generation, validation, deployment, and optimization.
//!
//! # Features
//!
//! - **Template Engine**: Generate Solana program code from templates
//! - **Architecture Validation**: Ensure compliance with AGENTS.md guidelines
//! - **Deployment Pipeline**: Build and deploy programs to validators
//! - **Story Automation**: Parse story files and suggest implementation steps
//! - **Optimization Analysis**: Find performance improvement opportunities
//!
//! # Examples
//!
//! ```no_run
//! use solana_toolbox::template::TemplateEngine;
//! use std::collections::HashMap;
//!
//! // Create a template engine
//! let engine = TemplateEngine::new().unwrap();
//!
//! // Render a template with substitutions
//! let mut vars = HashMap::new();
//! vars.insert("INSTRUCTION_NAME".to_string(), "Transfer".to_string());
//! let code = engine.load_and_render("instruction.rs.template", &vars).unwrap();
//! ```

pub mod commands;
pub mod template;
pub mod validation;
pub mod deployment;
pub mod story;
pub mod optimization;

pub use commands::*;
