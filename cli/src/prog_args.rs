use clap::{Parser, Subcommand};

/// Tool for managing mods for SSBU.
///
/// Focuses on doing two things well:
/// - Conflict resolution.
/// - Checking for mod updates.
#[derive(Debug, Parser)]
pub(crate) struct ProgArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Status,
    Add,
    Delete,
    Install,
    Update,
    EnableDisable,
    Resolve,
    CleanCache,
    SwitchCompare,
}
