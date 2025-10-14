mod config;
mod exec;
mod io_utils;
mod prompts;
mod rag;
mod verification;
mod agents {
    pub mod developer;
    pub mod repair;
    pub mod reviewer;
    pub use developer::extract_rust_block;
}

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use std::path::Path;

use crate::{
    agents::{developer::DeveloperAgent, repair::RepairAgent, reviewer::ReviewerAgent},
    config::{Cli, RefactorKind},
    exec::run_cmd,
    io_utils::{read_to_string, temp_rs_path, write_string},
    rag::load_few_shot,
    verification::{analyze_repo, cargo_check, cargo_test, clippy_check, fmt_check},
};

#[derive(Debug)]
enum State {
    Develop,
    UserVerify,
    Review,
    BuildAndTest,
    RepairLoop(u32),
    Done,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("mantra_rs=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();
    cli.validate()?;

    let repo = cli.repo.canonicalize()?;
    let file = cli.file.canonicalize()?;
    let model = cli.model.clone();

    for tool in ["rust-code-analysis-cli", "cargo", "rustfmt"] {
        if which::which(tool).is_err() {
            warn!("Tool `{}` not found on PATH; some checks may fail", tool);
        }
    }

    let original = read_to_string(&file)?;
    let analysis = analyze_repo(&repo, &file)
        .await
        .unwrap_or_else(|e| format!("analysis failed: {e}"));

    let few_shot = load_few_shot(&cli.rag_dir).unwrap_or_default();

    let dev = DeveloperAgent::new(&model);
    let reviewer = ReviewerAgent::new(&model);
    let repair = RepairAgent::new(&model);

    let mut state = State::Develop;
    let mut candidate_path = temp_rs_path(&file)?;
    let mut candidate_content = String::new();
    let mut last_logs = String::new();
    let mut repairs = 0u32;

    loop {
        match state {
            State::Develop => {
                info!("DeveloperAgent: generating candidate");
                let out = dev
                    .generate(
                        file.to_string_lossy().as_ref(),
                        &original,
                        &analysis,
                        &few_shot,
                        &cli.refactor_prompt,
                    )
                    .await?;
                write_string(&candidate_path, &out)?;
                candidate_content = out;
                state = State::UserVerify;
            }
            State::UserVerify => {
                if cli.yes {
                    state = State::Review;
                    continue;
                }
                let _ = run_cmd(
                    Path::new("/"),
                    "code",
                    &[
                        "-d",
                        file.to_str().unwrap(),
                        candidate_path.to_str().unwrap(),
                    ],
                )
                .await;
                eprintln!("\nOpen diff above. Apply this change to proceed? [y/N]: ");
                use std::io::stdin;
                let mut buf = String::new();
                stdin().read_line(&mut buf).ok();
                let ans = buf.trim().to_lowercase();
                if ans == "y" || ans == "yes" {
                    state = State::Review;
                } else {
                    info!("User rejected; exiting without changes.");
                    break;
                }
            }
            State::Review => {
                info!("ReviewerAgent: reviewing candidate");

                // Write candidate to file first so fmt/clippy can check the actual candidate
                write_string(&file, &candidate_content)?;

                // Run cargo fmt to auto-format the code
                info!("Running cargo fmt to clean up formatting...");
                let _ = run_cmd(&repo, "cargo", &["fmt"]).await;

                // Run cargo clippy --fix to auto-fix what it can
                info!("Running cargo clippy --fix to clean up linting issues...");
                let _ = run_cmd(&repo, "cargo", &["clippy", "--fix", "--allow-dirty", "--allow-staged"]).await;

                // Re-read the file after auto-fixes
                candidate_content = read_to_string(&file)?;


                let fmt_ok = fmt_check(&repo).await.unwrap_or(false);
                let clippy_ok = clippy_check(&repo).await.unwrap_or(false);

                let verdict = reviewer
                    .review(
                        cli.refactor_type.clone(),
                        &original,
                        &candidate_content,
                        &analysis,
                        fmt_ok,
                        clippy_ok,
                        None,
                    )
                    .await?;

                info!("Reviewer verdict: {}", verdict.verdict);
                info!("Reviewer reason: {:?}", verdict.reasons);
                if verdict.verdict == "revise" {
                    let mut augmented = cli.refactor_prompt.clone();
                    if let Some(g) = verdict.patch_guidance {
                        augmented.push_str("\nReviewer guidance:\n");
                        augmented.push_str(&g);
                    }
                    let out = dev
                        .generate(
                            file.to_string_lossy().as_ref(),
                            &original,
                            &analysis,
                            &few_shot,
                            &augmented,
                        )
                        .await?;
                    write_string(&candidate_path, &out)?;
                    candidate_content = out;
                    state = State::UserVerify;
                } else {
                    // File already written above, proceed to build and test
                    state = State::BuildAndTest;
                }
            }
            State::BuildAndTest => {
                let (ok_check, logs1) = cargo_check(&repo).await?;
                let (ok_test, logs2) = if ok_check {
                    cargo_test(&repo).await?
                } else {
                    (false, String::new())
                };
                last_logs = format!("{}\n{}", logs1, logs2);
                if ok_check && ok_test {
                    info!("Build & tests passed ✅");
                    state = State::Done;
                } else {
                    error!("Build/test failed; entering RepairAgent loop");
                    state = State::RepairLoop(0);
                }
            }
            State::RepairLoop(n) => {
                if n >= cli.max_repairs {
                    error!(
                        "Max repair attempts reached. Leaving candidate file at: {}",
                        file.display()
                    );
                    break;
                }
                repairs = n + 1;
                let repaired = repair
                    .repair(file.to_str().unwrap(), &candidate_content, &last_logs)
                    .await?;
                write_string(&file, &repaired)?;
                candidate_content = repaired;

                let (ok_check, logs1) = cargo_check(&repo).await?;
                let (ok_test, logs2) = if ok_check {
                    cargo_test(&repo).await?
                } else {
                    (false, String::new())
                };
                last_logs = format!("{}\n{}", logs1, logs2);

                if ok_check && ok_test {
                    info!("Repair attempt {} succeeded ✅", repairs);
                    state = State::Done;
                } else {
                    warn!("Repair attempt {} failed; retrying…", repairs);
                    state = State::RepairLoop(repairs);
                }
            }
            State::Done => {
                info!("Refactoring complete. File updated at {}", file.display());
                break;
            }
        }
    }

    Ok(())
}
