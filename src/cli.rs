use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "reflect", about = "Sync a claude --worktree to its repository root via Mutagen")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start syncing worktree to root
    Start {
        /// Path to worktree (default: auto-detect from CWD)
        worktree: Option<PathBuf>,
        /// Path to repository root (default: auto-detect from worktree)
        root: Option<PathBuf>,
        /// Stay in foreground and stream sync events
        #[arg(long)]
        watch: bool,
    },
    /// Stop syncing and restore root
    Stop {
        /// Path to repository root (default: auto-detect from CWD)
        root: Option<PathBuf>,
    },
    /// Show active sync sessions
    Status,
}
