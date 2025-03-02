//! Logic to identify which "asset" (eg skin slot, stage) this mod affects. Used to detect and resolve conflicts.
//!
//! Asset association is fully determined by the path of each file. It is always determined by:
//! - Whether a certain directory (eg. `fighter/yoshi`) or pattern (eg. `*/C02/*`) is present.
//! - If slot information appears in the actual file name itself (eg. `se_jack_c00.nus3audio`).
//!
//! The overall patterns to look for vary quite a bit between asset file types, so unfortunately the rules get pretty complicated.

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use ultimate_mod_man_rs_utils::types::ModId;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
enum ModType {
    CharacterSkin,
    StageSkin,
    Core,
}

/// What "asset" the mod file is associated with (eg. skin slot 02).
#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub enum ModFileAssetAssociation {
    CharSkinSlot(CharSkinSlotValue),
    Stage(StageSlotValue),

    /// File is not specific to any slot but may affect multiple slots.
    Global,

    /// File is present in the mod but has no effect on the game and can be safely ignored.
    NoEffect,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModFileInfo {
    mod_type: Vec<ModType>,
    owned_files: HashMap<ModFileAssetAssociation, Vec<Utf8PathBuf>>,
}

impl ModFileInfo {
    /// Crawl the mod directory looking for any files that we can classify.
    pub fn from_uncompressed_path(p: &Utf8Path) -> Self {
        todo!()
    }
}

#[derive(Debug)]
struct FileOwnerDb {
    mod_info: HashMap<ModId, ModFileInfo>,
    files_with_associations: HashMap<Utf8PathBuf, ModId>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct CharSkinSlotValue {
    char_key: String,
    skin_slot_idx: SkinSlotIdx,
}

impl Display for CharSkinSlotValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.char_key, self.skin_slot_idx)
    }
}

/// A skin slot can actually go beyond 0 - 7 where anything beyond `7` is used for special purposes. We still need to detect collisions in these ranges, although maybe we can have less strict logic for handling them.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SkinSlotIdx(u8);

impl Display for SkinSlotIdx {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            0..=7 => write!(f, "Skin slot")?,
            _ => write!(f, "Custom skin slot")?,
        };

        write!(f, " C{:2x} ({})", self.0, self.0)
    }
}

impl SkinSlotIdx {
    pub fn is_normal_skin_slot(&self) -> bool {
        matches!(self.0, 0..=7)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct StageSlotValue(u8);

#[cfg(test)]
mod tests {}
