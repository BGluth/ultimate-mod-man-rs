//! Mod manager state that is persisted between run. This includes downloaded mods, slot overrides, etc.
//!
//! The persisted mod manager state is stored in a directory where each mod gets it's own subdirectory. Each mod subdirectory looks like this:
//! mod_info.toml
//! mod_download_link_1_v_x_y_z (dir)
//!     \_ file_1, file_2, etc.
//! mod_download_link_2_v_x_y_z (dir)
//! mod_download_link_1_v_a_b_c (dir)
//! mod_download_link_2_v_a_b_c (dir)
//!
//! The idea is:
//! - Each mod on GameBanana has it's own unique ID and name.
//! - Each mod can have one or more download links for different variants of the mod.
//! - Each mod variant has it's own version (in most cases will likely be variant publish date).
//! - Any mod can have 1..* variants installed.
//! - Each mod variant can be enabled or disabled.
//! - A lock file is acquired before doing ANY reads/writes on the root mod directory.
//!
//! So on startup, the entire mod directory is read in and the installed mod structure is constructed.

use std::{
    collections::HashMap,
    convert::Infallible,
    fs::{self, create_dir_all},
    io,
    path::Path,
    str::FromStr,
};

use chrono::{DateTime, Utc};
use lockfile::Lockfile;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// static MOD_DB_FILE_NAME: &str = "mods.db";
// static MOD_DOWNLOAD_CACHE_DIR_NAME: &str = "download_cache";
// static DOWNLOAD_CACHE_ENTRY_INFO_FILE_NAME: &str = "cache_info";
static MOD_INFO_FILE_NAME: &str = "mod_info.toml";
static DOWNLOAD_CACHE_UNPACKED_DATA_DIR: &str = "data";
static DB_LOCKFILE_NAME: &str = ".lockfile";

pub type ModId = u64;

pub type LoadPersistedStateResult<T> = Result<T, LoadPersistedStateErr>;

#[derive(Debug, Error)]
pub enum LoadPersistedStateErr {
    #[error("No cache entry info for mod {0}")]
    MissingModCacheInfoEntry(String),

    #[error(transparent)]
    LockFileError(#[from] DBLockFileError),

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error(transparent)]
    DeserializationError(#[from] toml::de::Error),
}

type DBLockFileResult<T> = Result<T, DBLockFileError>;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct DBLockFileError(#[from] lockfile::Error);

// Need to ignore the unused field because we actually "use" this field when the struct gets dropped.
#[allow(dead_code)]
#[derive(Debug)]
struct DBLockFile(Lockfile);

impl DBLockFile {
    fn new(p: &Path) -> DBLockFileResult<Self> {
        Ok(Self(Lockfile::create(p.join(DB_LOCKFILE_NAME))?))
    }
}

#[derive(Debug)]
pub(crate) struct ModDb {
    directory_contents: ModDbDirectory,

    /// We hold the lock-file until the entire program exits.
    _lock_file: DBLockFile,
}

impl ModDb {
    pub(crate) fn load_from_path(p: &Path) -> LoadPersistedStateResult<Self> {
        if !p.exists() {
            info!("Data directory does not exist at \"{p:?}\". Creating...");
            create_dir_all(p)?;
        }

        let mut installed_mods = HashMap::new();

        // TODO: If there is a clean cross-platform way to access a in memory directory (eg. `/tmp` on Linux), place the lockfile there instead.
        let _lock_file = DBLockFile::new(p)?;

        for entry in fs::read_dir(p)? {
            let installed_mod_dir = entry?;

            // There should only be directories in the mod folder.
            if !installed_mod_dir.file_type()?.is_dir() {
                let unexpected_entry_name = installed_mod_dir.file_name();
                warn!(
                    "Found something other than a directory in the mod manager state directory at \"{p:?}\" ({unexpected_entry_name:?})"
                );
                continue;
            }

            if let Some(installed_mod) =
                InstalledModInfo::read_installed_mod_contents_dir(&installed_mod_dir.path())?
            {
                installed_mods.insert(installed_mod.id, installed_mod);
            }
        }

        Ok(Self {
            directory_contents: ModDbDirectory {
                entries: installed_mods,
            },
            _lock_file,
        })
    }

    pub(crate) fn get(&self, ident: &ModIdentifier) -> Option<&InstalledModInfo> {
        todo!()
    }

    pub(crate) fn get_mut(&mut self, ident: &ModIdentifier) -> Option<&mut InstalledModInfo> {
        todo!()
    }

    pub(crate) fn remove(&mut self, ident: &ModIdentifier) -> Option<InstalledModInfo> {
        todo!()
    }

    pub(crate) fn installed_mods(&self) -> impl Iterator<Item = &InstalledModInfo> {
        self.directory_contents.entries.values()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ModDbDirectory {
    entries: HashMap<ModId, InstalledModInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct InstalledModInfo {
    /// The ID of the mod on GameBanana.
    pub id: ModId,

    /// The name of the mod on GameBanana.
    pub name: String,

    /// Because there can be different variants available to download for a given mod, we need to also be able to specify which one we are using.
    pub installed_variants: Vec<InstalledVariant>,

    /// The version that we have in the mod manager.
    pub version: Option<String>,
}

impl InstalledModInfo {
    fn read_installed_mod_contents_dir(
        installed_mod_path: &Path,
    ) -> LoadPersistedStateResult<Option<Self>> {
        let mod_info_path = installed_mod_path.join(MOD_INFO_FILE_NAME);

        // To keep things simple (at least for now), we're going to assume that the directory structure inside each installed mod directory is always valid. If it's not, we are just going to warn the user at a minimum since validation is going to add a lot of complexity and should only happen if the user does manual intervention.
        if !mod_info_path.exists() {
            warn!(
                "Mod {mod_info_path:?} has no \"{MOD_INFO_FILE_NAME}\" file inside it's directory. This should never happen. Consider deleting this mod and adding again. Skipping..."
            );

            // TODO: Consider asking the user if we should remove this entry?

            return Ok(None);
        }

        let mod_info: InstalledModInfo = toml::from_str(&fs::read_to_string(&mod_info_path)?)?;

        // Quick simple verification check for the installed mod variants.
        for installed_variant in mod_info.installed_variants.iter() {
            let mod_variant_dir_path = mod_info_path.join(&installed_variant.name);

            if !mod_variant_dir_path.exists() {
                warn!(
                    "Found an installed mod variant that we do not actually have installed data for ({mod_variant_dir_path:?})! Will maybe add automatic resolution support in the future, but for now try deleting this mod and installing again. Skipping..."
                );
                return Ok(None);
            }
        }

        Ok(Some(mod_info))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CharacterSlotsAndOverrides {
    /// The name of the character. Keeping this dynamic in order to support custom characters and not just skins.
    char_name: String,

    /// The slots specified by the mod without changing them.
    mod_default_slots: Vec<SkinSlot>,

    /// In order to avoid conflicts (or if the user just wants a different slot), we can override the original slot to something else.
    slot_overrides: Vec<SlotOverride>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
struct SlotOverride {
    old: SkinSlot,
    new: SkinSlot,
}

/// Starting at `0` to avoid confusion just because the first slot in the game is `00`.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum SkinSlot {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

#[derive(Debug, Error)]
#[error("\"{0}\" is not a valid character slot")]
pub struct InvalidCharSlotErr(String);

impl FromStr for SkinSlot {
    type Err = InvalidCharSlotErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "00" => Ok(Self::Zero),
            "01" => Ok(Self::One),
            "02" => Ok(Self::Two),
            "03" => Ok(Self::Three),
            "04" => Ok(Self::Four),
            "05" => Ok(Self::Five),
            "06" => Ok(Self::Six),
            "07" => Ok(Self::Seven),
            _ => Err(InvalidCharSlotErr(s.to_string())),
        }
    }
}

/// Info that we can use to detect version changes.
///
/// There is really not much data available to detect version changes on GameBanana, so likely in most cases we are going to have to rely on file publish dates.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct ModVariantVersioningInfo {
    publish_date: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstalledVariant {
    /// The installed variant name is just the file name on GameBanana.   
    pub(crate) name: String,

    /// Slots used by the skin variant and any overrides.
    pub(crate) slots_and_overrides: CharacterSlotsAndOverrides,

    /// Whether or not the mod is enabled.
    pub(crate) enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ModIdentifier {
    /// The ID of the mod on Game Banana.
    Id(ModId),

    /// The name of the mod on Game Banana.
    Name(String),
}

/// Because mods can have multiple install payloads that affect multiple slots, we need a key type that identifies a mod and one of its specific variants.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ModIdentWithVariant {
    ident: ModIdentifier,
    variant_name: String,
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::mod_db::ModIdentifier;

    #[test]
    fn mod_identifier_from_name_works() {
        assert_eq!(ModIdentifier::from_str("./rust_mod").unwrap(), "./rust_mod");
    }

    #[test]
    fn mod_identifier_from_id_works() {
        assert_eq!(ModIdentifier::from_str("9001").unwrap(), 9001)
    }
}
