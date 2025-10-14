use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::{io::AsyncReadExt, process::Command};

#[derive(Debug, Clone)]
pub struct CmdOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub async fn run_cmd(cwd: &std::path::Path, program: &str, args: &[&str]) -> Result<CmdOutput> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(cwd);

    let mut child = cmd.spawn().with_context(|| format!("spawn {}", program))?;
    let status = child.wait().await?.code().unwrap_or(-1);

    let mut out = String::new();
    let mut err = String::new();

    if let Some(mut o) = child.stdout.take() {
        let mut buf = String::new();
        o.read_to_string(&mut buf).await.ok();
        out = buf;
    }
    if let Some(mut e) = child.stderr.take() {
        let mut buf = String::new();
        e.read_to_string(&mut buf).await.ok();
        err = buf;
    }

    Ok(CmdOutput {
        status,
        stdout: out,
        stderr: err,
    })
}
