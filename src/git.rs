use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context};

pub struct WorktreeContext {
    pub worktree: PathBuf,
    pub root: PathBuf,
}

pub fn detect_from_cwd() -> anyhow::Result<WorktreeContext> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let git_path = cwd.join(".git");

    if git_path.is_file() {
        let contents = fs::read_to_string(&git_path)
            .context("Failed to read .git file")?;
        let gitdir = parse_gitdir(&contents)?;
        let root = strip_worktree_suffix(&gitdir)?;
        return Ok(WorktreeContext { worktree: cwd, root });
    }

    if git_path.is_dir() {
        return Err(anyhow!(
            "Not inside a git worktree — this looks like a repository root.\n\
             Run this from a claude --worktree session,\n\
             or specify paths manually: reflect start <worktree> <root>"
        ));
    }

    Err(anyhow!("Not inside a git repository"))
}

fn parse_gitdir(contents: &str) -> anyhow::Result<PathBuf> {
    let line = contents
        .lines()
        .find(|l| l.starts_with("gitdir:"))
        .ok_or_else(|| anyhow!("No gitdir: line found in .git file"))?;

    let path_str = line
        .strip_prefix("gitdir:")
        .unwrap()
        .trim();

    Ok(PathBuf::from(path_str))
}

fn strip_worktree_suffix(gitdir: &Path) -> anyhow::Result<PathBuf> {
    // gitdir = /path/to/root/.git/worktrees/branch-name
    // walk up: branch-name → worktrees/ → .git/ → root/
    gitdir
        .parent()                          // .git/worktrees/
        .and_then(|p| p.parent())          // .git/
        .and_then(|p| p.parent())          // root/
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("Could not determine main repo root from gitdir: {}", gitdir.display()))
}

pub fn stash_push(root: &Path) -> anyhow::Result<bool> {
    let output = Command::new("git")
        .args(["stash", "push", "-m", "reflect-backup"])
        .current_dir(root)
        .output()
        .context("Failed to run git stash")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git stash failed: {}", stderr));
    }

    // "No local changes to save" means nothing was stashed
    Ok(!stdout.contains("No local changes"))
}

pub fn restore_working_tree(root: &Path) -> anyhow::Result<()> {
    let output = Command::new("git")
        .args(["checkout", "."])
        .current_dir(root)
        .output()
        .context("Failed to run git checkout")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git checkout . failed: {}", stderr));
    }

    Ok(())
}

pub fn stash_pop(root: &Path) -> anyhow::Result<()> {
    // Find the reflect-backup stash
    let output = Command::new("git")
        .args(["stash", "list"])
        .current_dir(root)
        .output()
        .context("Failed to list git stashes")?;

    let list = String::from_utf8_lossy(&output.stdout);
    let stash_ref = list
        .lines()
        .find(|l| l.contains("reflect-backup"))
        .and_then(|l| l.split(':').next())
        .map(|s| s.to_string());

    if let Some(ref_name) = stash_ref {
        let output = Command::new("git")
            .args(["stash", "pop", &ref_name])
            .current_dir(root)
            .output()
            .context("Failed to pop git stash")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("git stash pop failed: {}", stderr));
        }
    }

    Ok(())
}
