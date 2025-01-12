use std::{
    convert::Infallible,
    env::current_dir,
    fmt::{self, Display},
    ops::Deref,
    path::PathBuf,
    str::FromStr,
};

use clap::{Args, Parser, Subcommand};
use log::warn;
use ultimate_mod_man_rs_lib::mod_db::ModId;

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
    pub(crate) state_dir_path: DisplayablePathBuf,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Get the status of all installed mods or information on one or more specific mods. Do not specify any mods to get generic stats.
    Status(StatusArgs),

    /// Add new mods to the manager.
    Add(AddArgs),

    /// Delete mods added to the manager.  
    Delete,

    /// Install mods directly to the Switch.
    Install(InstallArgs),

    /// Check if updates are available for any added mods.
    CheckForUpdates,

    /// Update the mods installed on a Switch with the mods that added to the manager.
    UpdateInstalled,

    /// Enable or disable a given set of mods.
    EnableDisable(EnableDisableArgs),

    /// Resolve any conflicts identified by the mod manager.
    Resolve,

    /// Swap character slots used by a mod.
    ChangeSlot,

    /// Purge the cache of downloaded mods.
    CleanCache,

    /// Compare the state of a mod to state of the mod installed on a Switch.
    SwitchCompare,
}

#[derive(Args, Debug)]
pub(crate) struct StatusArgs {
    #[command(flatten)]
    pub(crate) mods: ModIdentifiersList,
}

#[derive(Args, Debug)]
pub(crate) struct AddArgs {
    #[command(flatten)]
    pub(crate) mods: ModIdentifiersList,
}

#[derive(Args, Debug)]
pub(crate) struct InstallArgs {
    #[arg(short = 'i', long)]
    install_path: PathBuf,
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
    pub(crate) mods: Vec<ModIdentifier>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ModIdentifier {
    /// The ID of the mod on Game Banana.
    Id(ModId),

    /// The name of the mod on Game Banana.
    Name(String),
}

impl PartialEq<ModId> for ModIdentifier {
    fn eq(&self, other_id: &ModId) -> bool {
        matches!(self, ModIdentifier::Id(other) if other_id == other)
    }
}

impl PartialEq<&str> for ModIdentifier {
    fn eq(&self, other_name: &&str) -> bool {
        matches!(self, ModIdentifier::Name(other) if other_name == other)
    }
}

impl FromStr for ModIdentifier {
    // Impossible for this conversion to fail.
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // If we can parse it as a ID (`u64`), then treat it as an ID. Otherwise just assume that we received the mod name.
        s.parse::<u64>()
            .map(ModIdentifier::Id)
            .or_else(|_| Ok(ModIdentifier::Name(s.to_string())))
    }
}

fn get_os_default_state_dir_path() -> DisplayablePathBuf {
    // TODO: Unwrap for now. Not sure how to handle `Result`s in default Clap args...
    match dirs::cache_dir() {
        Some(p) => p.into(),
        None => {
            warn!(
                "Unable to find a config directory for this OS! Using the current directory instead as a fallback, but this should be considered a bug and be reported to the maintainers."
            );
            current_dir().unwrap().into()
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DisplayablePathBuf(PathBuf);

impl Display for DisplayablePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<PathBuf> for DisplayablePathBuf {
    fn from(v: PathBuf) -> Self {
        Self(v)
    }
}

impl FromStr for DisplayablePathBuf {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = PathBuf::from_str(s)?;
        Ok(Self(res))
    }
}

impl Deref for DisplayablePathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::ModIdentifier;

    #[test]
    fn mod_identifier_from_name_works() {
        assert_eq!(ModIdentifier::from_str("./rust_mod").unwrap(), "./rust_mod");
    }

    #[test]
    fn mod_identifier_from_id_works() {
        assert_eq!(ModIdentifier::from_str("9001").unwrap(), 9001)
    }
}
