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
use derive_builder::Builder;
use lockfile::Lockfile;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::download_artifact_parser::SkinSlot;

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

    pub(crate) fn get(&self, id: &ModId) -> Option<&InstalledModInfo> {
        todo!()
    }

    pub(crate) fn get_mut(&mut self, id: &ModId) -> Option<&mut InstalledModInfo> {
        todo!()
    }

    pub(crate) fn remove(&mut self, id: &ModId) -> Option<InstalledModInfo> {
        todo!()
    }

    pub(crate) fn add_mod(&mut self, id: ModId) {
        if let Some(entry) = self.directory_contents.entries.get(&id) {
            info!(
                "Mod {} (id: {}) is already installed. Ignoring...",
                entry.name, id
            );
        }
    }

    pub(crate) fn delete_mod(&mut self, id: ModId) {
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
    char_and_default_slots: SkinSlot,

    /// In order to avoid conflicts (or if the user just wants a different slot), we can override the original slot to something else.
    slot_overrides: Vec<SlotOverride>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
struct SlotOverride {
    old: SkinSlot,
    new: SkinSlot,
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

impl From<ModId> for ModIdentifier {
    fn from(v: ModId) -> Self {
        Self::Id(v)
    }
}

impl From<String> for ModIdentifier {
    fn from(v: String) -> Self {
        Self::Name(v)
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

#[derive(Debug, Error)]
pub enum ModWithVariantIdentifierStrError {
    #[error("Missing \"/\" when parsing the mod variant and identifier string \"{0}\"")]
    MissingSlashSeparator(String),

    #[error(
        "Missing \"/\" variant (left hand side of \"/\" when parsing the mod variant and identifier string \"{0}\""
    )]
    MissingVariant(String),
}

/// Because mods can have multiple install payloads that affect multiple slots, we need a key type that identifies a mod and one of its specific variants.
#[derive(Builder, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ModWithVariantIdentifier {
    #[builder(setter(custom = true))]
    ident: ModIdentifier,
    variant_name: String,
}

impl ModWithVariantIdentifierBuilder {
    pub fn ident_id(&mut self, id: ModId) -> &mut Self {
        self.ident = Some(id.into());
        self
    }

    pub fn ident_name(&mut self, name: String) -> &mut Self {
        self.ident = Some(name.into());
        self
    }
}

impl FromStr for ModWithVariantIdentifier {
    type Err = ModWithVariantIdentifierStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // (ident/variant_name)
        // eg. (9001/super_cool_variant_2)

        // Note:
        // - `split()` is guaranteed to always have at least one element.
        // `ModIdentifier::from_str` is infallible.
        let mut split_str = s.split("/");
        let ident = ModIdentifier::from_str(split_str.next().unwrap()).unwrap();

        let variant_name = match split_str.next() {
            None => {
                return Err(ModWithVariantIdentifierStrError::MissingSlashSeparator(
                    s.to_string(),
                ));
            }
            Some(s) if s.is_empty() => {
                return Err(ModWithVariantIdentifierStrError::MissingVariant(
                    s.to_string(),
                ));
            }
            Some(s) => s.to_string(),
        };

        Ok(Self {
            ident,
            variant_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::mod_db::{
        ModIdentifier, ModWithVariantIdentifier, ModWithVariantIdentifierBuilder,
        ModWithVariantIdentifierStrError,
    };

    use super::ModId;

    #[test]
    fn mod_identifier_from_name_string_works() {
        assert_eq!(ModIdentifier::from_str("./rust_mod").unwrap(), "./rust_mod");
    }

    #[test]
    fn mod_identifier_from_id_string_works() {
        assert_eq!(ModIdentifier::from_str("9001").unwrap(), 9001)
    }

    fn m_var_ident_from_str(str: &str) -> ModWithVariantIdentifier {
        ModWithVariantIdentifier::from_str(str).unwrap()
    }

    fn m_var_ident_test_payload_from_id(id: ModId, var_name: &str) -> ModWithVariantIdentifier {
        ModWithVariantIdentifierBuilder::default()
            .ident_id(id)
            .variant_name(var_name.to_string())
            .build()
            .unwrap()
    }

    fn m_var_ident_test_payload_from_name(name: &str, var_name: &str) -> ModWithVariantIdentifier {
        ModWithVariantIdentifierBuilder::default()
            .ident_name(name.to_string())
            .variant_name(var_name.to_string())
            .build()
            .unwrap()
    }

    #[test]
    fn mod_identifier_with_variant_string_works() {
        assert_eq!(
            m_var_ident_from_str("9001/super_cool_variant_2"),
            m_var_ident_test_payload_from_id(9001, "super_cool_variant_2")
        );

        assert_eq!(
            m_var_ident_from_str("./."),
            m_var_ident_test_payload_from_name(".", ".")
        );

        assert_eq!(
            m_var_ident_from_str("my_cool_rust_mod/release_v_3_4_8"),
            m_var_ident_test_payload_from_name("my_cool_rust_mod", "release_v_3_4_8")
        );
    }

    #[test]
    fn mod_identifier_with_variant_string_errors_on_invalid_input() {
        println!("{:?}", ModWithVariantIdentifier::from_str(""));

        assert!(matches!(
            ModWithVariantIdentifier::from_str(""),
            Err(ModWithVariantIdentifierStrError::MissingSlashSeparator(_))
        ));

        assert!(matches!(
            ModWithVariantIdentifier::from_str("just_a_mod_nameModWithVariantIdentifier::from_str"),
            Err(ModWithVariantIdentifierStrError::MissingSlashSeparator(_))
        ));

        assert!(matches!(
            ModWithVariantIdentifier::from_str("just_a_mod_name/"),
            Err(ModWithVariantIdentifierStrError::MissingVariant(_))
        ));
    }
}
