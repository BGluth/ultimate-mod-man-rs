//! Mod manager state that is persisted between run. This includes downloaded
//! mods, slot overrides, etc.
//!
//! The persisted mod manager state is stored in a directory where each mod gets
//! it's own subdirectory. Each mod subdirectory looks like this: mod_info.toml
//! mod_download_link_1_v_x_y_z (dir)
//!     \_ file_1, file_2, etc.
//! mod_download_link_2_v_x_y_z (dir)
//! mod_download_link_1_v_a_b_c (dir)
//! mod_download_link_2_v_a_b_c (dir)
//!
//! The idea is:
//! - Each mod on GameBanana has it's own unique ID and name.
//! - Each mod can have one or more download links for different variants of the
//!   mod.
//! - Each mod variant has it's own version (in most cases will likely be
//!   variant publish date).
//! - Any mod can have 1..* variants installed.
//! - Each mod variant can be enabled or disabled.
//! - A lock file is acquired before doing ANY reads/writes on the root mod
//!   directory.
//!
//! So on startup, the entire mod directory is read in and the installed mod
//! structure is constructed.

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
    download_artifact_parser::{ModPayloadParseInfo, VariantParseError},
    mod_file_classifier::{ModFileAssetAssociation, VariantFileInfo},
};
use ultimate_mod_man_rs_utils::{
    types::{
        AssetSlot, AvailableSlotsToSwapToInfo, CharSkinSlotValue, ModId,
        PickedNonSwappableResolutionOption, PickedResolutionOption, SkinSlotIdx, SkinSlotValue,
        StageSlotIdx, StageSlotValue, SwappableAssetSlot, VariantAndId,
    },
    user_input_delegate::VariantConflictSummary,
    utils::{DeserializationError, deserialize_data_from_path},
};

use crate::in_prog_action::{Action, InProgAction, InProgActionError};

pub type ModDbResult<T> = Result<T, ModDbError>;

#[derive(Debug, Error)]
pub enum ModDbError {
    #[error(transparent)]
    VariantParseError(#[from] VariantParseError),

    #[error(transparent)]
    DeserializationError(#[from] DeserializationError),

    #[error(transparent)]
    LockFileError(#[from] DBLockFileError),

    #[error(transparent)]
    InProgActionError(#[from] InProgActionError),

    #[error(transparent)]
    IoError(#[from] io::Error),
}

static MOD_INFO_FILE_NAME: &str = "mod_info.toml";
static EXPANDED_MOD_INFO_DIR_NAME: &str = "expanded";
static DOWNLOAD_CACHE_UNPACKED_DATA_DIR: &str = "data";
static IN_PROG_ACTION_FILE_NAME: &str = "in_prog_action.toml";
static DB_LOCKFILE_NAME: &str = ".lockfile";

type DBLockFileResult<T> = Result<T, DBLockFileError>;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct DBLockFileError(#[from] lockfile::Error);

// TODO: Add pending writes to the lock file to get transaction like behavior...
// Need to ignore the unused field because we actually "use" this field when the
// struct gets dropped.
#[allow(dead_code)]
#[derive(Debug)]
struct DBLockFile(Lockfile);

impl DBLockFile {
    fn new(p: &Utf8Path) -> DBLockFileResult<Self> {
        Ok(Self(Lockfile::create(p.join(DB_LOCKFILE_NAME))?))
    }
}

#[derive(Debug)]
pub(crate) enum UnableToEnableReason {
    Conflicts(VariantConflictInfo),
    AlreadyEnabled,
}

#[derive(Debug)]
pub struct VariantConflictInfo {
    pub key: VariantAndId,
    pub conflicts: Vec<ConflictingModVariant>,
}

#[derive(Debug)]
pub struct ConflictingModVariant {
    /// Key of the mod variant with unresolved conflicts.
    pub key: VariantAndId,

    /// All slots that currently have a conflict.
    pub slots: Vec<AssetSlot>,
}

#[derive(Debug)]
pub(crate) struct ModDb {
    // persisted_state:
    directory_contents: ModDbDirectory,

    mod_file_associations: EnabledModFileAssociations,

    /// We hold the lock-file until the entire program exits.
    _lock_file: DBLockFile,
}

impl ModDb {
    pub(crate) fn load_from_path(p: &Utf8Path) -> ModDbResult<Self> {
        if !p.exists() {
            info!("Data directory does not exist at \"{p:?}\". Creating...");
            create_dir_all(p)?;
        }

        let mut mod_file_associations = EnabledModFileAssociations::new();
        let mut installed_mods = HashMap::new();

        // TODO: If there is a clean cross-platform way to access a in memory directory
        // (eg. `/tmp` on Linux), place the lockfile there instead.
        let _lock_file = DBLockFile::new(p)?;

        for entry in Utf8Path::read_dir_utf8(p)? {
            let installed_mod_dir = entry?;

            // There should only be directories in the mod folder.
            if !installed_mod_dir.file_type()?.is_dir() {
                let unexpected_entry_name = installed_mod_dir.file_name();
                warn!(
                    "Found something other than a directory in the mod manager state directory at \
                     \"{p:?}\" ({unexpected_entry_name:?})"
                );

                continue;
            }

            // We are assuming that any serialized enabled mods do not conflict with each
            // other, since we should only serialize mods that have no conflicts. This will
            // panic if that's not the case.
            if let Some(mut installed_mod) =
                InstalledModInfo::read_installed_mod_contents_dir(installed_mod_dir.path())?
            {
                for var_info in installed_mod.installed_variants.values_mut() {
                    if mod_file_associations
                        .add_mod_info_to_global_lookup(&var_info.file_info)
                        .is_some()
                    {
                        warn!(
                            "Mod variant ({}) that was serialized as enabled is unable to be \
                             enabled due to conflicts. This is either a bug in the manager or the \
                             serialized data may have been modified outside of this tool. \
                             Disabling...",
                            var_info.name
                        )
                    }

                    var_info.enabled = false;
                }

                installed_mods.insert(installed_mod.id, installed_mod);
            }
        }

        Ok(Self {
            directory_contents: ModDbDirectory {
                dir_path: p.into(),
                entries: installed_mods,
            },
            mod_file_associations,
            _lock_file,
        })
    }

    pub(crate) fn journal_action_as_in_prog(&self, action: Action) -> ModDbResult<()> {
        let in_prog_action_file_path = self.directory_contents.get_in_prog_action_path();
        assert!(!fs::exists(&in_prog_action_file_path)?);

        let in_prog = InProgAction::new(action);
        in_prog.sync_to_disk(&in_prog_action_file_path)?;

        Ok(())
    }

    pub(crate) fn get_in_prog_action_if_any(&self) -> ModDbResult<Option<InProgAction>> {
        let in_prog_action_file_path = self.directory_contents.get_in_prog_action_path();
        let res = InProgAction::load_from_disk_if_present(&in_prog_action_file_path)?;

        Ok(res)
    }

    pub(crate) fn remove_in_prog_action(&self) -> ModDbResult<()> {
        let in_prog_action_file_path = self.directory_contents.get_in_prog_action_path();
        fs::remove_file(in_prog_action_file_path)?;

        Ok(())
    }

    pub(crate) fn add_variant(
        &mut self,
        key: &VariantAndId,
        payload: ScrapedBananaModData,
    ) -> ModDbResult<()> {
        let mod_dir_path = self.directory_contents.get_path_to_mod(key.id);

        let compressed_path = self.add_compressed_archive(
            &mod_dir_path,
            &key.variant_name,
            &payload.variant_download_artifact,
        )?;

        let mod_info_path = mod_dir_path.join(MOD_INFO_FILE_NAME);

        // Load mod info from disk.
        let mut mod_info: InstalledModInfo = match fs::exists(mod_info_path)? {
            false => InstalledModInfo::new(key.id, payload.mod_name, payload.version),
            true => deserialize_data_from_path(&mod_dir_path.join(MOD_INFO_FILE_NAME))?,
        };

        mod_info.add_variant(key.variant_name.clone(), mod_dir_path, compressed_path)?;

        Ok(())
    }

    /// It's pretty annoying, but we need to write the compressed archive to
    /// disk in some cases (looking at `unrar`) before we can parse it.
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

    pub(crate) fn get_variant(&self, key: &VariantAndId) -> Option<&InstalledVariant> {
        todo!()
    }

    pub(crate) fn exists(&self, key: &VariantAndId) -> bool {
        self.get_variant(key).is_some()
    }

    /// Removes a variant from the database. This also removes all files
    /// associated with the variant.
    ///
    /// Because we need to handle the case where a remove may have not fully
    /// completed and only do part of the work, we need to keep going if we
    /// discover that some state is not there (eg. variant doesn't exist in
    /// database).
    pub(crate) fn remove_variant(
        &mut self,
        key: &VariantAndId,
    ) -> ModDbResult<Option<InstalledVariant>> {
        let mod_info = self.directory_contents.get_mod_mut_expected(key.id);

        let var_info = mod_info.installed_variants.remove(&key.variant_name);
        if var_info.is_none() {
            warn!(
                "Tried removing a mod variant that was not installed! ({})",
                key
            );
        }

        // Check if the directory exists anyways.
        let mod_variant_path = self.directory_contents.get_path_to_mod_variant(key);
        if fs::exists(&mod_variant_path)? {
            fs::remove_dir_all(&mod_variant_path)?;
        }

        Ok(var_info)
    }

    pub(crate) fn remove_mod(&mut self, id: &ModId) -> Option<InstalledModInfo> {
        todo!()
    }

    pub(crate) fn installed_mods(&self) -> impl Iterator<Item = &InstalledModInfo> {
        self.directory_contents.entries.values()
    }

    /// Like `remove_variant` except it expects that things may be randomly
    /// missing.
    pub(crate) fn cleanup_traces_of_variant(&mut self, key: &VariantAndId) -> ModDbResult<()> {
        todo!()
    }

    /// Like `remove_mod` except it expects that things may be randomly missing.
    pub(crate) fn cleanup_traces_of_mod(&mut self, key: &ModId) -> ModDbResult<()> {
        todo!()
    }

    /// Attempts to enable the mod. If it can not be enabled, the reason along
    /// with additional info will be returned.
    pub(crate) fn enable_variant(
        &mut self,
        key: &VariantAndId,
    ) -> ModDbResult<Option<UnableToEnableReason>> {
        let var_info = self.directory_contents.get_variant_mut_expected(key);

        if var_info.enabled {
            return Ok(Some(UnableToEnableReason::AlreadyEnabled));
        }

        if let Some(conflicts) = self
            .mod_file_associations
            .add_mod_info_to_global_lookup(&var_info.file_info)
        {
            return Ok(Some(UnableToEnableReason::Conflicts(conflicts)));
        }

        var_info.enabled = true;

        Ok(None)
    }

    pub(crate) fn disable_variant(&mut self, key: VariantAndId) {
        let var_info = self.directory_contents.get_variant_mut_expected(&key);
        debug_assert!(
            !var_info.enabled,
            "Tried disabled a mod variant that was already disabled! ({})",
            key
        );

        var_info.enabled = false;
    }

    pub(crate) fn get_variant_conflict_summary(
        &self,
        key: &VariantAndId,
    ) -> Option<VariantConflictSummary> {
        todo!()
    }

    pub(crate) fn resolve_conflict(
        &self,
        conflict: &ConflictingModVariant,
    ) -> ModDbConflictResolver {
        todo!()
    }

    pub(crate) fn detect_any_overrides_that_are_no_longer_needed(
        &self,
    ) -> Vec<(VariantAndId, VariantOverride)> {
        todo!()
    }

    pub(crate) fn get_available_slots_to_swap_to(
        &self,
        slot: &SwappableAssetSlot,
    ) -> AvailableSlotsToSwapToInfo {
        todo!()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ModDbDirectory {
    dir_path: Utf8PathBuf,
    entries: HashMap<ModId, InstalledModInfo>,
}

impl ModDbDirectory {
    fn get_mod_mut_expected(&mut self, key: ModId) -> &mut InstalledModInfo {
        self.entries
            .get_mut(&key)
            .unwrap_or_else(|| panic!("Expected to have a mod for mod ID {}", key))
    }

    fn get_variant_mut_expected(&mut self, key: &VariantAndId) -> &mut InstalledVariant {
        let mod_info = self.get_mod_mut_expected(key.id);
        mod_info.get_variant_mut_expected(&key.variant_name)
    }

    fn get_path_to_mod(&self, id: ModId) -> Utf8PathBuf {
        let mod_name = &self.get_mod_name_expected(id);
        self.dir_path.join(get_mod_directory_name(id, mod_name))
    }

    fn get_path_to_mod_variant(&self, key: &VariantAndId) -> Utf8PathBuf {
        let mod_name = &self.get_mod_name_expected(key.id);
        self.dir_path.join(get_path_section_from_key(key, mod_name))
    }

    fn get_mod_name_expected(&self, id: ModId) -> &str {
        self.entries
            .get(&id)
            .expect(
                "Missing mod entry when constructing path to it's directory! This should never \
                 happen!",
            )
            .name
            .as_str()
    }

    fn get_in_prog_action_path(&self) -> Utf8PathBuf {
        self.dir_path.join(IN_PROG_ACTION_FILE_NAME)
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

    /// Because there can be different variants available to download for a
    /// given mod, we need to also be able to specify which one we are using.
    pub installed_variants: HashMap<String, InstalledVariant>,

    // TODO: Determine if the mod itself or the variant should hold the version info...
    /// The version that we have in the mod manager.
    pub version: Option<String>,
}

impl InstalledModInfo {
    fn new(id: ModId, name: String, version: Option<String>) -> Self {
        Self {
            id,
            name,
            installed_variants: HashMap::default(),
            version,
        }
    }

    fn add_variant(
        &mut self,
        var_name: String,
        mod_dir_path: Utf8PathBuf,
        compressed_path: Utf8PathBuf,
    ) -> ModDbResult<()> {
        let mod_variant_path = mod_dir_path.join(&var_name);
        fs::create_dir(&mod_variant_path)?;

        let expanded_mod_dir_path = mod_variant_path.join(EXPANDED_MOD_INFO_DIR_NAME);
        fs::create_dir(&expanded_mod_dir_path)?;

        let parse_info = ModPayloadParseInfo::new(&compressed_path)?;
        parse_info.expand_archive_to_disk(&expanded_mod_dir_path)?;

        let variant_file_info = VariantFileInfo::from_uncompressed_path(&expanded_mod_dir_path);

        let installed_var = InstalledVariant::new(var_name.clone(), variant_file_info);
        self.installed_variants.insert(var_name, installed_var);

        Ok(())
    }

    fn get_variant_mut_expected(&mut self, key: &str) -> &mut InstalledVariant {
        self.installed_variants
            .get_mut(key)
            .unwrap_or_else(|| panic!("Expected to have a variant for the key {}", key))
    }
}

impl InstalledModInfo {
    fn read_installed_mod_contents_dir(installed_mod_path: &Utf8Path) -> ModDbResult<Option<Self>> {
        let mod_info_path: Utf8PathBuf = installed_mod_path.join(MOD_INFO_FILE_NAME);

        // To keep things simple (at least for now), we're going to assume that the
        // directory structure inside each installed mod directory is always valid. If
        // it's not, we are just going to warn the user at a minimum since validation is
        // going to add a lot of complexity and should only happen if the user does
        // manual intervention.
        if !mod_info_path.exists() {
            warn!(
                "Mod {mod_info_path:?} has no \"{MOD_INFO_FILE_NAME}\" file inside it's \
                 directory. This should never happen. Consider deleting this mod and adding \
                 again. Skipping..."
            );

            // TODO: Consider asking the user if we should remove this entry?

            return Ok(None);
        }

        let mod_info: InstalledModInfo = deserialize_data_from_path(&mod_info_path)?;

        // Quick simple verification check for the installed mod variants.
        for installed_variant_name in mod_info.installed_variants.keys() {
            let mod_variant_dir_path = mod_info_path.join(installed_variant_name);

            if !mod_variant_dir_path.exists() {
                warn!(
                    "Found an installed mod variant that we do not actually have installed data \
                     for ({mod_variant_dir_path:?})! Will maybe add automatic resolution support \
                     in the future, but for now try deleting this mod and installing again. \
                     Skipping..."
                );
                return Ok(None);
            }
        }

        Ok(Some(mod_info))
    }
}

/// Info that we can use to detect version changes.
///
/// There is really not much data available to detect version changes on
/// GameBanana, so likely in most cases we are going to have to rely on file
/// publish dates.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct ModVariantVersioningInfo {
    publish_date: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstalledVariant {
    // TODO: Duplicate data now that these entries are keyed with this in a `HashMap`?
    /// The installed variant name is just the file name on GameBanana.   
    pub(crate) name: String,

    /// Metadata describing the file associations between files and mod assets.
    pub(crate) file_info: VariantFileInfo,

    /// Overrides in use by the variant.
    pub(crate) overrides: Vec<VariantOverride>,

    /// Whether or not the mod is enabled.
    pub(crate) enabled: bool,
}

impl InstalledVariant {
    fn new(name: String, file_info: VariantFileInfo) -> Self {
        Self {
            name,
            file_info,
            overrides: Vec::default(),
            enabled: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) enum VariantOverride {
    CharacterSkin(Override<SkinSlotValue>),
    StageSkin(Override<StageSlotValue>),
    Global(Override<Utf8PathBuf>),
}

/// A slot for a variant that has been overridden. This contains the mapping of
/// the original slot to the set slot.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Override<T> {
    old: T,
    new: T,
}

/// We also need to perform "global" lookups to detect conflicts. Specifically,
/// we need to be able to quickly detect if a given file in a mod is already
/// occupied with another mod.
///
/// This is constructed each time during startup and is not serialized (may
/// change in the future).
///
/// Also note that this lookup contains all mod files after overrides have been
/// applied.
#[derive(Debug)]
struct EnabledModFileAssociations {
    association_lookup: HashMap<ModFileAssetAssociation, ModId>,
}

impl EnabledModFileAssociations {
    fn new() -> Self {
        todo!()
    }

    /// Unless the serialized state is manipulated, enabling a mod should never
    /// have any conflicts. If it does, then it will not be added to this.
    fn add_mod_info_to_global_lookup(
        &mut self,
        var_info: &VariantFileInfo,
    ) -> Option<VariantConflictInfo> {
        todo!()
    }

    fn get_any_mod_associated_with_asset(
        &self,
        assoc_type: &ModFileAssetAssociation,
    ) -> Option<ModId> {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct ModDbConflictResolver<'a> {
    db: &'a mut ModDb,
    pending_changes: HashMap<AssetSlot, AssetSlotChange>,
    conflicts_remaining: Vec<AssetSlot>,
}

impl<'a> ModDbConflictResolver<'a> {
    pub(crate) fn get_next_conflict_to_resolve(&mut self) -> Option<AssetConflict> {
        todo!()
    }

    pub(crate) fn resolve_conflict(&mut self, resolution: PickedResolutionOption) {
        todo!()
    }

    pub(crate) fn commit(self) {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) enum AssetSlotChange {
    CharacterSkin(SkinSlotIdx),
    StageSkin(StageSlotIdx),
    Global(Utf8PathBuf),
}

/// Stores information about a conflict and options to resolve it.
#[derive(Debug)]
pub(crate) enum AssetConflict {
    Swappable(SwappableAssetConflict),
    NonSwappable(NonSwappableAssetConflict),
}

impl AssetConflict {
    pub(crate) fn slot(&self) -> &AssetSlot {
        todo!()
    }
}

#[derive(Debug)]
pub enum NonSwappableAssetConflict {
    Stage(StageSlotConflict),
    Global(GlobalConflict),
}

#[derive(Debug)]
pub enum SwappableAssetConflict {
    CharacterSkin(CharacterSkinConflict),
}

impl SwappableAssetConflict {
    pub(crate) fn existing(&self) -> SwappableAssetSlot {
        match self {
            SwappableAssetConflict::CharacterSkin(character_skin_conflict) => {
                SwappableAssetSlot::CharacterSkin(character_skin_conflict.existing.clone())
            },
        }
    }

    pub(crate) fn slots_available_to_swap_into(&self) -> AvailableSlotsToSwapToInfo {
        match self {
            // TODO: Remove clone once we figure out API...
            Self::CharacterSkin(character_skin_conflict) => {
                AvailableSlotsToSwapToInfo::CharacterSkin(
                    character_skin_conflict.possible_resolutions.clone(),
                )
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct CharacterSkinConflict {
    existing: CharSkinSlotValue,
    possible_resolutions: Vec<SkinSlotValue>,
}

impl CharacterSkinConflict {
    fn resolve(self, res: PickedNonSwappableResolutionOption) -> CharSkinSlotResolution {
        todo!()
    }
}

pub(crate) struct CharSkinSlotResolution {
    res: PickedResolutionOption,
}

#[derive(Debug)]
pub(crate) struct StageSlotConflict {
    existing: StageSlotValue,
}

impl StageSlotConflict {
    pub(crate) fn resolve(
        self,
        res: PickedNonSwappableResolutionOption,
    ) -> StageSkinSlotResolution {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct StageSkinSlotResolution {
    res: PickedNonSwappableResolutionOption,
}

#[derive(Debug)]
pub(crate) struct GlobalConflict {
    existing: Utf8PathBuf,
}

impl GlobalConflict {
    pub(crate) fn resolve(self, res: PickedNonSwappableResolutionOption) -> GlobalResolution {
        todo!()
    }
}

pub(crate) struct GlobalResolution {
    res: PickedNonSwappableResolutionOption,
}
