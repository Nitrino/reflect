use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context};
use sha2::{Digest, Sha256};

pub fn session_name(root: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(root.to_string_lossy().as_bytes());
    let hash = hasher.finalize();
    format!("reflect-{}", hex::encode(&hash[..4]))
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

pub fn check_installed() -> anyhow::Result<()> {
    which::which("mutagen").map_err(|_| {
        anyhow!(
            "Mutagen not found. Install with:\n  \
             brew install mutagen-io/mutagen/mutagen   # macOS\n  \
             https://mutagen.io/documentation/introduction/installation"
        )
    })?;
    Ok(())
}

pub fn create_session(name: &str, worktree: &Path, root: &Path) -> anyhow::Result<()> {
    let output = Command::new("mutagen")
        .args([
            "sync",
            "create",
            &format!("--name={}", name),
            "--sync-mode=one-way-replica",
            "--ignore-vcs",
            "--ignore=target/",
            "--ignore=node_modules/",
            "--ignore=.claude/",
        ])
        .arg(worktree.to_string_lossy().as_ref())
        .arg(root.to_string_lossy().as_ref())
        .output()
        .context("Failed to run mutagen sync create")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("mutagen sync create failed: {}", stderr));
    }

    Ok(())
}

pub fn terminate_session(name: &str) -> anyhow::Result<()> {
    let output = Command::new("mutagen")
        .args(["sync", "terminate", name])
        .output()
        .context("Failed to run mutagen sync terminate")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("mutagen sync terminate failed: {}", stderr));
    }

    Ok(())
}

pub fn session_exists(name: &str) -> bool {
    Command::new("mutagen")
        .args(["sync", "list", name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn list_reflect_sessions() -> anyhow::Result<String> {
    let output = Command::new("mutagen")
        .args(["sync", "list"])
        .output()
        .context("Failed to run mutagen sync list")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    // Filter to only reflect-* sessions
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("reflect-"))
        .collect();

    if lines.is_empty() {
        return Ok("No active reflect sessions.".to_string());
    }

    Ok(stdout)
}

pub fn monitor_session(name: &str) -> anyhow::Result<()> {
    let mut child = Command::new("mutagen")
        .args(["sync", "monitor", name])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to run mutagen sync monitor")?;

    child.wait().context("mutagen sync monitor exited unexpectedly")?;
    Ok(())
}
