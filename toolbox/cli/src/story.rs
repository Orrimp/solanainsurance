//! Story parsing and implementation utilities
//!
//! This module parses story files from `stories/` directory and extracts tasks,
//! helping automate story-driven development workflows.
//!
//! # File Structure
//!
//! ```text
//! stories/
//!   story-001/
//!     story.md    # Story description
//!     tasks.md    # Task checklist
//! ```
//!
//! # Examples
//!
//! ```no_run
//! use solana_toolbox::story::parse_story;
//!
//! let story = parse_story("001").unwrap();
//! println!("Story: {}", story.title);
//! for task in &story.tasks {
//!     println!("  {} {}", if task.completed { "✅" } else { "⬜" }, task.description);
//! }
//! ```

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;

/// Represents a user story
///
/// Contains story metadata and associated tasks.
///
/// # Examples
///
/// ```no_run  
/// use solana_toolbox::story::{Story, Task};
///
/// let story = Story {
///     id: "001".to_string(),
///     title: "Implement Token Transfer".to_string(),
///     status: "in-progress".to_string(),
///     description: "Add token transfer functionality".to_string(),
///     tasks: vec![],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Story {
    pub id: String,
    pub title: String,
    pub status: String,
    pub description: String,
    pub tasks: Vec<Task>,
}

/// Represents a task within a story
///
/// # Examples
///
/// ```
/// use solana_toolbox::story::Task;
///
/// let task = Task {
///     id: "1".to_string(),
///     description: "Create state struct".to_string(),
///     completed: false,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub completed: bool,
}

/// Parse a story from the stories directory
///
/// Reads `stories/story-{id}/story.md` and `tasks.md` to extract story information.
///
/// # Arguments
///
/// * `story_id` - Story ID (e.g., "001")
///
/// # Errors
///
/// Returns an error if:
/// - Story directory doesn't exist
/// - story.md file is missing
/// - File parsing fails
///
/// # Examples
///
/// ```no_run
/// use solana_toolbox::story::parse_story;
///
/// let story = parse_story("001").unwrap();
/// println!("Story: {}", story.title);
/// println!("Tasks: {}", story.tasks.len());
/// ```
pub fn parse_story(story_id: &str) -> Result<Story> {
    let story_dir = PathBuf::from(format!("stories/story-{}", story_id));
    
    if !story_dir.exists() {
        anyhow::bail!("Story directory not found: {}", story_dir.display());
    }

    let story_file = story_dir.join("story.md");
    let tasks_file = story_dir.join("tasks.md");

    if !story_file.exists() {
        anyhow::bail!("Story file not found: {}", story_file.display());
    }

    let story_content = fs::read_to_string(&story_file)
        .context("Failed to read story.md")?;

    // Parse title (first H1)
    let title_re = Regex::new(r"^#\s+(.+)$").unwrap();
    let title = story_content
        .lines()
        .find_map(|line| title_re.captures(line))
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| format!("Story {}", story_id));

    // Parse status
    let status_re = Regex::new(r"(?i)Status:\s*(.+?)(?:\s*—|$)").unwrap();
    let status = status_re
        .captures(&story_content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    // Extract description (everything until "## User Stories" or similar)
    let mut description = String::new();
    let mut in_header = false;
    for line in story_content.lines() {
        if line.starts_with("## ") {
            break;
        }
        if !line.starts_with('#') || in_header {
            description.push_str(line);
            description.push('\n');
            in_header = true;
        }
    }

    // Parse tasks if tasks.md exists
    let mut tasks = Vec::new();
    if tasks_file.exists() {
        let tasks_content = fs::read_to_string(&tasks_file)
            .context("Failed to read tasks.md")?;
        tasks = parse_tasks(&tasks_content)?;
    }

    Ok(Story {
        id: story_id.to_string(),
        title,
        status,
        description: description.trim().to_string(),
        tasks,
    })
}

/// Parse tasks from tasks.md content
fn parse_tasks(content: &str) -> Result<Vec<Task>> {
    let task_re = Regex::new(r"^-\s+\[([ xX])\]\s+\*\*(.+?)\*\*\s+(.+)$").unwrap();
    let mut tasks = Vec::new();

    for line in content.lines() {
        if let Some(caps) = task_re.captures(line) {
            let completed = caps.get(1).unwrap().as_str().to_lowercase() == "x";
            let id = caps.get(2).unwrap().as_str().to_string();
            let description = caps.get(3).unwrap().as_str().to_string();

            tasks.push(Task {
                id,
                description,
                completed,
            });
        }
    }

    Ok(tasks)
}

/// List all available stories
pub fn list_stories() -> Result<Vec<String>> {
    let stories_dir = Path::new("stories");
    
    if !stories_dir.exists() {
        return Ok(Vec::new());
    }

    let mut story_ids = Vec::new();
    
    for entry in fs::read_dir(stories_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if dir_name.starts_with("story-") {
                    if let Some(id) = dir_name.strip_prefix("story-") {
                        story_ids.push(id.to_string());
                    }
                }
            }
        }
    }

    story_ids.sort();
    Ok(story_ids)
}

/// Extract instruction names mentioned in a story
pub fn extract_instructions_from_story(story: &Story) -> Vec<String> {
    let instruction_re = Regex::new(r"`([A-Z][a-zA-Z]+)`").unwrap();
    let mut instructions = Vec::new();

    // Search in description
    for cap in instruction_re.captures_iter(&story.description) {
        if let Some(m) = cap.get(1) {
            let name = m.as_str();
            // Filter out common non-instruction words
            if name != "Status" && name != "Auth" && name != "Dev" {
                instructions.push(name.to_string());
            }
        }
    }

    // Search in task descriptions
    for task in &story.tasks {
        for cap in instruction_re.captures_iter(&task.description) {
            if let Some(m) = cap.get(1) {
                let name = m.as_str();
                if name != "Status" && name != "Auth" && name != "Dev" {
                    instructions.push(name.to_string());
                }
            }
        }
    }

    // Deduplicate
    instructions.sort();
    instructions.dedup();
    instructions
}

/// Print a story summary
pub fn print_story_summary(story: &Story) {
    println!("{}", "═".repeat(50).bright_cyan());
    println!("{} {}", "  Story:".bright_cyan().bold(), story.title);
    println!("{}", "═".repeat(50).bright_cyan());
    println!();
    
    println!("{} {}", "ID:".bright_blue(), story.id.bright_yellow());
    
    let status_colored = match story.status.as_str() {
        s if s.contains("DONE") || s.contains("✅") => s.bright_green(),
        s if s.contains("WIP") || s.contains("IN PROGRESS") => s.bright_yellow(),
        s if s.contains("TODO") => s.bright_red(),
        s => s.normal(),
    };
    println!("{} {}", "Status:".bright_blue(), status_colored);
    println!();

    if !story.tasks.is_empty() {
        println!("{}", "Tasks:".bright_blue());
        let total = story.tasks.len();
        let completed = story.tasks.iter().filter(|t| t.completed).count();
        
        for task in &story.tasks {
            let checkbox = if task.completed { "✅" } else { "⬜" };
            println!("   {} {} {}", checkbox, task.id.bright_yellow(), task.description);
        }
        
        println!();
        println!("{} {}/{} completed", "Progress:".bright_blue(), completed, total);
    }
}
