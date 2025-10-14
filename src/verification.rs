use crate::config::RefactorKind;
use crate::exec::run_cmd;
use anyhow::{Context, Result};
use regex::Regex;
use tracing::info;
use std::path::Path;

/// Run rust-code-analysis-cli to snapshot structure (best-effort).
pub async fn analyze_repo(repo: &Path, file: &Path) -> Result<String> {
    let args = [
        "-m",
        "functions,modules,structs,impls",
        "-p",
        file.to_str().unwrap(),
    ];
    let out = run_cmd(repo, "rust-code-analysis-cli", &args)
        .await
        .context("rust-code-analysis-cli failed")?;
    Ok(format!("STDOUT:\n{}\nSTDERR:\n{}", out.stdout, out.stderr))
}

/// rustfmt --check
pub async fn fmt_check(repo: &Path) -> Result<bool> {
    let out = run_cmd(repo, "cargo", &["fmt", "--", "--check"]).await?;
    Ok(out.status == 0)
}

/// clippy -D warnings
pub async fn clippy_check(repo: &Path) -> Result<bool> {
    let out = run_cmd(repo, "cargo", &["clippy", "-q", "--", "-D", "warnings"]).await?;
    Ok(out.status == 0)
}

pub async fn cargo_check(repo: &Path) -> Result<(bool, String)> {
    let out = run_cmd(repo, "cargo", &["check"]).await?;
    let logs = format!("{}\n{}", out.stdout, out.stderr);
    Ok((out.status == 0, logs))
}

pub async fn cargo_test(repo: &Path) -> Result<(bool, String)> {
    let out = run_cmd(repo, "cargo", &["test", "--all", "--quiet"]).await?;
    let logs = format!("{}\n{}", out.stdout, out.stderr);
    Ok((out.status == 0, logs))
}

/// Very light heuristic verification per refactor type on the single file.
pub fn verify_refactor_heuristic(kind: RefactorKind, original: &str, candidate: &str) -> bool {
    match kind {
        RefactorKind::ExtractMethod => {
            // More lenient regex that matches functions anywhere (not just at line start)
            let re = Regex::new(r"(?m)\bfn\s+\w+\s*\(").unwrap();
            let orig_count = re.find_iter(original).count();
            let cand_count = re.find_iter(candidate).count();
            let n_new = cand_count.saturating_sub(orig_count);

            info!("Failed to verify refactor: {:?}, number of functions did not change after refactoring (from {} to {}).", kind, orig_count, cand_count);

            // For extract method: should have at least one new function
            // Remove the strict length requirement since refactoring might make code shorter
            n_new > 0
        }
        RefactorKind::InlineMethod => {
            let re = Regex::new(r"(?m)\bfn\s+\w+\s*\(").unwrap();
            re.find_iter(candidate).count() < re.find_iter(original).count()
        }
        RefactorKind::MoveMethod => candidate.len() + 20 < original.len(),
        RefactorKind::RenameMethod => original != candidate,
    }
}
