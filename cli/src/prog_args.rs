use std::env::current_dir;

use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand};
use log::warn;
use ultimate_mod_man_rs_utils::types::VariantAndIdentifier;

/// Tool for managing mods for SSBU.
///
/// Focuses on doing two things well:
/// - Conflict resolution.
/// - Checking for mod updates.
#[derive(Debug, Parser)]
#[clap(verbatim_doc_comment)]
#[command(author, version)]
pub(crate) struct ProgArgs {
    #[command(subcommand)]
    pub(crate) command: Command,

    /// Path to the directory where the mod manager state and cache is located.
    #[arg(short = 'p', long, default_value_t = get_os_default_state_dir_path())]
    pub(crate) state_dir_path: Utf8PathBuf,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Get the status of all installed mods or information on one or more specific mods. Do not specify any mods to get generic stats.
    Status(StatusCliArgs),

    /// Add new mods to the manager.
    Add(AddArgs),

    /// Delete mods added to the manager.  
    Delete,

    /// Check if updates are available for any added mods.
    CheckForUpdates,

    /// Update the mods installed on a Switch with the mods that added to the manager.
    SyncWithSwitch,

    /// Enable or disable a given set of mods.
    EnableDisable(EnableDisableArgs),

    /// Resolve any conflicts identified by the mod manager.
    ResolveConflicts,

    /// Swap character slots used by a mod.
    ChangeSlot,

    /// Compare the state of a mod to state of the mod installed on a Switch.
    SwitchCompare,
}

#[derive(Args, Debug)]
pub(crate) struct StatusCliArgs {
    #[command(flatten)]
    pub(crate) mods: ModIdentifiersList,
}

#[derive(Args, Debug)]
pub(crate) struct AddArgs {
    #[command(flatten)]
    pub(crate) mods: ModIdentifiersList,
}

#[derive(Args, Debug)]
pub(crate) struct InstallToSwitchArgs {
    #[arg(short = 'i', long)]
    install_path: Utf8PathBuf,
}

#[derive(Args, Debug)]
pub(crate) struct EnableDisableArgs {
    /// List of mods to enable or disable.
    #[command(flatten)]
    pub(crate) mods: ModIdentifiersList,

    /// Enable or disable all of the given mods.
    #[arg(short = 'e', long, default_value_t = true)]
    pub(crate) enable: bool,
}

/// Struct is purely just to wrap the Clippy docs in order to avoid duplicating them.
#[derive(Args, Clone, Debug)]
pub(crate) struct ModIdentifiersList {
    /// A list of mods to work with.
    ///
    /// Each mod can be specified with either:
    /// - The ID on GameBanana
    /// - The name of the mod on GameBanana.
    #[clap(verbatim_doc_comment)]
    pub(crate) mods: Vec<VariantAndIdentifier>,
}

fn get_os_default_state_dir_path() -> Utf8PathBuf {
    // TODO: Unwrap for now. Not sure how to handle `Result`s in default Clap args...
    let res = match dirs::cache_dir() {
        Some(p) => p,
        None => {
            warn!(
                "Unable to find a config directory for this OS! Using the current directory instead as a fallback, but this should be considered a bug and be reported to the maintainers."
            );
            current_dir().unwrap()
        }
    };

    Utf8PathBuf::from_path_buf(res).unwrap()
}
