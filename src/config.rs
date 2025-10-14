use anyhow::{ensure, Result};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Refactoring kinds we “verify” heuristically for Rust.
#[derive(Debug, Clone, ValueEnum)]
pub enum RefactorKind {
    ExtractMethod,
    InlineMethod,
    MoveMethod,
    RenameMethod,
}

#[derive(Parser, Debug)]
#[command(
    name = "mantra-rs",
    version,
    about = "MANTRA-style multi-agent refactoring for Rust"
)]
pub struct Cli {
    /// Path to the Rust repository root (must contain Cargo.toml)
    #[arg(long)]
    pub repo: PathBuf,

    /// Path to the Rust file to refactor (relative or absolute)
    #[arg(long)]
    pub file: PathBuf,

    /// Free-form human instruction paired with the refactor type (the “what/why”)
    #[arg(long)]
    pub refactor_prompt: String,

    /// Refactoring type (used by Reviewer for verification heuristics)
    #[arg(long, value_enum)]
    pub refactor_type: RefactorKind,

    /// Optional model (default: gpt-4o-mini)
    #[arg(long, default_value = "gpt-4o-mini")]
    pub model: String,

    /// Optional path holding few-shot examples for RAG
    #[arg(long, default_value = "rag_examples")]
    pub rag_dir: PathBuf,

    /// Max repair attempts
    #[arg(long, default_value_t = 10)]
    pub max_repairs: u32,

    /// Non-interactive (auto-apply without VS Code diff prompt)
    #[arg(long, default_value_t = false)]
    pub yes: bool,
}

impl Cli {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.repo.join("Cargo.toml").exists(),
            "No Cargo.toml found in --repo"
        );
        ensure!(self.file.exists(), "--file does not exist");
        Ok(())
    }
}
