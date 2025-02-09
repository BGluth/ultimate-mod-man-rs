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
    fs::{self, create_dir_all},
    io,
};

use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Utc};
use lockfile::Lockfile;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ultimate_mod_man_rs_scraper::{
    banana_scraper::ScrapedBananaModData,
    download_artifact_parser::{ModPayloadParseInfo, SkinSlot, VariantParseError},
};
use ultimate_mod_man_rs_utils::types::{ModId, VariantAndId};

pub type ModDbResult<T> = Result<T, ModDbError>;

#[derive(Debug, Error)]
pub enum ModDbError {
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error(transparent)]
    VariantParseError(#[from] VariantParseError),

    #[error(transparent)]
    DeserializationError(#[from] toml::de::Error),
}

static MOD_INFO_FILE_NAME: &str = "mod_info.toml";
static EXPANDED_MOD_INFO_DIR_NAME: &str = "expanded";
static DOWNLOAD_CACHE_UNPACKED_DATA_DIR: &str = "data";
static DB_LOCKFILE_NAME: &str = ".lockfile";

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

// TODO: Add pending writes to the lock file to get transaction like behavior...
// Need to ignore the unused field because we actually "use" this field when the struct gets dropped.
#[allow(dead_code)]
#[derive(Debug)]
struct DBLockFile(Lockfile);

impl DBLockFile {
    fn new(p: &Utf8Path) -> DBLockFileResult<Self> {
        Ok(Self(Lockfile::create(p.join(DB_LOCKFILE_NAME))?))
    }
}

#[derive(Debug)]
pub(crate) struct ModDb {
    directory_contents: ModDbDirectory,

    /// We hold the lock-file until the InstalledModInfoentire program exits.
    _lock_file: DBLockFile,
}

impl ModDb {
    pub(crate) fn load_from_path(p: &Utf8Path) -> LoadPersistedStateResult<Self> {
        if !p.exists() {
            info!("Data directory does not exist at \"{p:?}\". Creating...");
            create_dir_all(p)?;
        }

        let mut installed_mods = HashMap::new();

        // TODO: If there is a clean cross-platform way to access a in memory directory (eg. `/tmp` on Linux), place the lockfile there instead.
        let _lock_file = DBLockFile::new(p)?;

        for entry in Utf8Path::read_dir_utf8(p)? {
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
                InstalledModInfo::read_installed_mod_contents_dir(installed_mod_dir.path())?
            {
                installed_mods.insert(installed_mod.id, installed_mod);
            }
        }

        Ok(Self {
            directory_contents: ModDbDirectory {
                dir_path: p.into(),
                entries: installed_mods,
            },
            _lock_file,
        })
    }

    pub(crate) fn add(
        &mut self,
        key: &VariantAndId,
        payload: ScrapedBananaModData,
    ) -> ModDbResult<()> {
        let mod_dir_path = self.directory_contents.get_path_to_mod(key.id);
        let mod_variant_path = mod_dir_path.join(&key.variant_name);

        fs::create_dir(&mod_variant_path)?;

        let compressed_path = self.add_compressed_archive(
            &mod_dir_path,
            &key.variant_name,
            &payload.variant_download_artifact,
        )?;

        let mod_info_path = mod_dir_path.join(MOD_INFO_FILE_NAME);

        // Load mod info from disk.
        let mut mod_info: InstalledModInfo = match fs::exists(mod_info_path)? {
            false => InstalledModInfo::new(key.id, payload.mod_name, payload.version),
            true => toml::from_str(&fs::read_to_string(mod_dir_path.join(MOD_INFO_FILE_NAME))?)?,
        };

        mod_info.add_variant(key.variant_name.clone());

        let expanded_mod_dir_path = mod_variant_path.join(EXPANDED_MOD_INFO_DIR_NAME);
        fs::create_dir(&expanded_mod_dir_path)?;

        let parse_info = ModPayloadParseInfo::new(&compressed_path)?;
        parse_info.expand_archive_to_disk(&expanded_mod_dir_path)?;

        Ok(())
    }

    /// It's pretty annoying, but we need to write the compressed archive to disk in some cases (looking at `unrar`) before we can parse it.
    fn add_compressed_archive(
        &mut self,
        mod_dir_path: &Utf8Path,
        variant_name: &str,
        compressed_payload: &[u8],
    ) -> ModDbResult<Utf8PathBuf> {
        let mod_artifact_path = mod_dir_path.join(variant_name);
        fs::write(&mod_artifact_path, compressed_payload)?;

        Ok(mod_artifact_path)
    }

    pub(crate) fn get(&self, key: &VariantAndId) -> Option<&InstalledModInfo> {
        todo!()
    }

    pub(crate) fn get_mut(&mut self, key: &VariantAndId) -> Option<&mut InstalledModInfo> {
        todo!()
    }

    pub(crate) fn exists(&self, key: &VariantAndId) -> bool {
        todo!()
    }

    pub(crate) fn remove(&mut self, key: &VariantAndId) -> Option<InstalledVariant> {
        todo!()
    }

    pub(crate) fn remove_mod(&mut self, id: &ModId) -> Option<InstalledModInfo> {
        todo!()
    }

    pub(crate) fn installed_mods(&self) -> impl Iterator<Item = &InstalledModInfo> {
        self.directory_contents.entries.values()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ModDbDirectory {
    dir_path: Utf8PathBuf,
    entries: HashMap<ModId, InstalledModInfo>,
}

impl ModDbDirectory {
    fn get_path_to_mod(&self, id: ModId) -> Utf8PathBuf {
        let mod_name = &self.get_mod_name_expected(id);
        self.dir_path.join(get_mod_directory_name(id, mod_name))
    }

    fn get_path_to_mod_variant(&self, key: &VariantAndId) -> Utf8PathBuf {
        let mod_name = &self.get_mod_name_expected(key.id);
        self.dir_path.join(get_path_section_from_key(key, mod_name))
    }

    fn get_mod_name_expected(&self, id: ModId) -> &str {
        self.entries.get(&id).expect("Missing mod entry when constructing path to it's directory! This should never happen!").name.as_str()
    }
}

fn get_path_section_from_key(key: &VariantAndId, mod_name: &str) -> Utf8PathBuf {
    let mod_dir_name = get_mod_directory_name(key.id, mod_name);
    format!("{}/{}", mod_dir_name, key.variant_name).into()
}

fn get_mod_directory_name(id: ModId, mod_name: &str) -> String {
    format!("{}_{}", mod_name, id)
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
    fn new(id: ModId, name: String, version: Option<String>) -> Self {
        Self {
            id,
            name,
            installed_variants: Vec::default(),
            version,
        }
    }

    fn add_variant(&mut self, name: String) {
        todo!()
    }
}

impl InstalledModInfo {
    fn read_installed_mod_contents_dir(
        installed_mod_path: &Utf8Path,
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

impl InstalledVariant {
    fn new(name: String) -> Self {
        Self {
            name,
            slots_and_overrides: todo!(),
            enabled: todo!(),
        }
    }
}
