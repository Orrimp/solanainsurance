//! Template loading and rendering utilities
//!
//! This module provides a template engine for generating Solana program code from
//! templates with placeholder substitution.
//!
//! # Examples
//!
//! ```no_run
//! use solana_toolbox::template::{TemplateEngine, pascal_to_snake, snake_to_pascal};
//! use std::collections::HashMap;
//!
//! // Create engine and load template
//! let engine = TemplateEngine::new().unwrap();
//! let mut vars = HashMap::new();
//! vars.insert("INSTRUCTION_NAME".to_string(), "Transfer".to_string());
//! vars.insert("SNAKE_NAME".to_string(), "transfer".to_string());
//!
//! let code = engine.load_and_render("instruction.rs.template", &vars).unwrap();
//!
//! // Convert naming conventions
//! assert_eq!(pascal_to_snake("Transfer"), "transfer");
//! assert_eq!(snake_to_pascal("transfer"), "Transfer");
//! ```

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Template engine for code generation
///
/// Loads templates from `toolbox/templates/` directory and performs
/// placeholder substitution using `{{PLACEHOLDER}}` syntax.
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::template::TemplateEngine;
/// use std::collections::HashMap;
///
/// let engine = TemplateEngine::new().unwrap();
/// let mut vars = HashMap::new();
/// vars.insert("NAME".to_string(), "Example".to_string());
/// let result = engine.load_and_render("test.rs.template", &vars).unwrap();
/// ```
pub struct TemplateEngine {
    template_dir: PathBuf,
}

impl TemplateEngine {
    /// Create a new template engine
    ///
    /// # Errors
    ///
    /// Returns an error if the `toolbox/templates/` directory does not exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use solana_toolbox::template::TemplateEngine;
    ///
    /// let engine = TemplateEngine::new().unwrap();
    /// ```
    pub fn new() -> Result<Self> {
        let template_dir = PathBuf::from("toolbox/templates");
        if !template_dir.exists() {
            anyhow::bail!("Template directory not found: {}", template_dir.display());
        }
        Ok(Self { template_dir })
    }

    /// Load a template from file
    ///
    /// # Arguments
    ///
    /// * `template_name` - Name of the template file (e.g., "instruction.rs.template")
    ///
    /// # Errors
    ///
    /// Returns an error if the template file cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use solana_toolbox::template::TemplateEngine;
    ///
    /// let engine = TemplateEngine::new().unwrap();
    /// let template = engine.load("test.rs.template").unwrap();
    /// assert!(template.contains("{{TEST_NAME}}"));
    /// ```
    pub fn load(&self, template_name: &str) -> Result<String> {
        let template_path = self.template_dir.join(template_name);
        fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to load template: {}", template_path.display()))
    }

    /// Render a template with the given substitutions
    ///
    /// Replaces all `{{PLACEHOLDER}}` patterns with their corresponding values.
    ///
    /// # Arguments
    ///
    /// * `template` - The template string containing placeholders
    /// * `substitutions` - Map of placeholder names to replacement values
    ///
    /// # Examples
    ///
    /// ```
    /// use solana_toolbox::template::TemplateEngine;
    /// use std::collections::HashMap;
    ///
    /// let engine = TemplateEngine::new().unwrap();
    /// let template = "Hello {{NAME}}!";
    /// let mut vars = HashMap::new();
    /// vars.insert("NAME".to_string(), "World".to_string());
    /// let result = engine.render(template, &vars);
    /// assert_eq!(result, "Hello World!");
    /// ```
    pub fn render(&self, template: &str, substitutions: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        
        // Replace all {{PLACEHOLDER}} patterns
        for (key, value) in substitutions {
            let pattern = format!("{{{{{}}}}}", key);
            result = result.replace(&pattern, value);
        }
        
        result
    }

    /// Load and render a template in one step
    pub fn load_and_render(
        &self,
        template_name: &str,
        substitutions: &HashMap<String, String>,
    ) -> Result<String> {
        let template = self.load(template_name)?;
        Ok(self.render(&template, substitutions))
    }
}

/// Convert PascalCase to snake_case
///
/// # Examples
///
/// ```
/// use solana_toolbox::template::pascal_to_snake;
///
/// assert_eq!(pascal_to_snake("Transfer"), "transfer");
/// assert_eq!(pascal_to_snake("HTTPSConnection"), "https_connection");
/// assert_eq!(pascal_to_snake("SimpleTest"), "simple_test");
/// ```
pub fn pascal_to_snake(s: &str) -> String {
    let re = Regex::new(r"([A-Z])").unwrap();
    let result = re.replace_all(s, "_$1").to_lowercase();
    result.trim_start_matches('_').to_string()
}

/// Convert snake_case to PascalCase
///
/// # Examples
///
/// ```
/// use solana_toolbox::template::snake_to_pascal;
///
/// assert_eq!(snake_to_pascal("transfer"), "Transfer");
/// assert_eq!(snake_to_pascal("https_connection"), "HttpsConnection");
/// assert_eq!(snake_to_pascal("simple_test"), "SimpleTest");
/// ```
pub fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Format field definitions for instruction data
///
/// Converts field specifications like "amount:u64" into Rust struct field definitions.
///
/// # Arguments
///
/// * `fields` - Slice of field specifications in "name:type" format
///
/// # Examples
///
/// ```
/// use solana_toolbox::template::format_fields;
///
/// let fields = vec![
///     "amount:u64".to_string(),
///     "recipient:Pubkey".to_string(),
/// ];
/// let formatted = format_fields(&fields);
/// assert!(formatted.contains("pub amount: u64,"));
/// assert!(formatted.contains("pub recipient: Pubkey,"));
/// ```
pub fn format_fields(fields: &[String]) -> String {
    if fields.is_empty() {
        return String::new();
    }
    
    fields
        .iter()
        .map(|field| {
            let parts: Vec<&str> = field.split(':').collect();
            if parts.len() == 2 {
                format!("    pub {}: {},", parts[0], parts[1])
            } else {
                format!("    // Invalid field: {}", field)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Ensure output directory exists
pub fn ensure_output_dir(dir: &Path) -> Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pascal_to_snake() {
        assert_eq!(pascal_to_snake("Initialize"), "initialize");
        assert_eq!(pascal_to_snake("Transfer"), "transfer");
        assert_eq!(pascal_to_snake("UpdateState"), "update_state");
    }

    #[test]
    fn test_snake_to_pascal() {
        assert_eq!(snake_to_pascal("initialize"), "Initialize");
        assert_eq!(snake_to_pascal("transfer"), "Transfer");
        assert_eq!(snake_to_pascal("update_state"), "UpdateState");
    }

    #[test]
    fn test_format_fields() {
        let fields = vec![
            "amount".to_string() + ":u64",
            "recipient".to_string() + ":Pubkey",
        ];
        let result = format_fields(&fields);
        assert!(result.contains("pub amount: u64"));
        assert!(result.contains("pub recipient: Pubkey"));
    }
}
