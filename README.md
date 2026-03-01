<p align="center">
  <img src="banner.avif" alt="reflect banner" />
</p>

# reflect

One-way sync from git worktrees to repo root via Mutagen.

## Install

```bash
# Shell installer (macOS/Linux)
curl --proto '=https' -LsSf https://github.com/Nitrino/reflect/releases/latest/download/reflect-installer.sh | sh

# Homebrew
brew install Nitrino/tap/reflect

# From source
cargo install --path .
```

Requires [Mutagen](https://mutagen.io):

```bash
brew install mutagen-io/mutagen/mutagen
```

## Usage

```bash
# From inside a git worktree:
reflect start           # auto-detect worktree and root
reflect start --watch   # same + live sync output (Ctrl+C to stop)
reflect stop            # stop sync, restore root
reflect status          # list active sessions

# Manual override:
reflect start <worktree> <root>
reflect stop <root>
```

## How it works

1. **Start** — stashes any uncommitted changes in the root repo, then creates a one-way Mutagen sync session (worktree → root)
2. **Stop** — terminates the Mutagen session and pops the stash to restore the root

Session names are deterministic hashes of the root path — no config or state files needed. Mutagen owns all session state.

## Auto-detection

When run from a git worktree, `reflect` reads the `.git` file to find the main repository root automatically. No arguments required.
