mod cli;
mod git;
mod mutagen;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::anyhow;
use clap::Parser;

use cli::{Cli, Command};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Start { worktree, root, watch } => cmd_start(worktree, root, watch),
        Command::Stop { root } => cmd_stop(root),
        Command::Status => cmd_status(),
    }
}

fn cmd_start(worktree: Option<PathBuf>, root: Option<PathBuf>, watch: bool) -> anyhow::Result<()> {
    let (worktree, root) = match (worktree, root) {
        (Some(w), Some(r)) => (w.canonicalize()?, r.canonicalize()?),
        (None, None) => {
            let ctx = git::detect_from_cwd()?;
            (ctx.worktree, ctx.root)
        }
        _ => return Err(anyhow!("Provide both <worktree> and <root>, or neither for auto-detection")),
    };

    mutagen::check_installed()?;

    let name = mutagen::session_name(&root);

    if mutagen::session_exists(&name) {
        return Err(anyhow!(
            "Session already running for {}\nUse: reflect stop",
            root.display()
        ));
    }

    let stashed = git::stash_push(&root, &name)?;
    if stashed {
        eprintln!("Stashed root changes (reflect-backup)");
    }

    mutagen::create_session(&name, &worktree, &root)?;

    eprintln!("Reflecting\n  {} -> {}", worktree.display(), root.display());

    if watch {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        let stop_root = root.clone();
        let stop_name = name.clone();

        ctrlc::set_handler(move || {
            if r.swap(false, Ordering::SeqCst) {
                eprintln!("\nStopping...");
                let _ = mutagen::terminate_session(&stop_name);
                let _ = git::restore_working_tree(&stop_root);
                let _ = git::stash_pop(&stop_root, &stop_name);
                eprintln!("Root restored. Reflect stopped.");
                std::process::exit(0);
            }
        }).ok();

        mutagen::monitor_session(&name)?;
    }

    Ok(())
}

fn cmd_stop(root: Option<PathBuf>) -> anyhow::Result<()> {
    let root = match root {
        Some(r) => r.canonicalize()?,
        None => git::detect_from_cwd()?.root,
    };

    let name = mutagen::session_name(&root);

    mutagen::terminate_session(&name)?;
    git::restore_working_tree(&root)?;
    git::stash_pop(&root, &name)?;

    eprintln!("Stopped. Root restored.");
    Ok(())
}

fn cmd_status() -> anyhow::Result<()> {
    mutagen::check_installed()?;
    let output = mutagen::list_reflect_sessions()?;
    println!("{}", output);
    Ok(())
}
