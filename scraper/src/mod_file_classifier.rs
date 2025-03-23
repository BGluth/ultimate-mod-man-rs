//! Logic to identify which "asset" (eg skin slot, stage) this mod affects. Used
//! to detect and resolve conflicts.
//!
//! Asset association is fully determined by the path of each file. It is always
//! determined by:
//! - Whether a certain directory (eg. `fighter/yoshi`) or pattern (eg.
//!   `*/C02/*`) is present.
//! - If slot information appears in the actual file name itself (eg.
//!   `se_jack_c00.nus3audio`).
//!
//! The overall patterns to look for vary quite a bit between asset file types,
//! so unfortunately the rules get pretty complicated.

use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use ultimate_mod_man_rs_utils::types::{CharSkinSlotValue, ModId, StageSlotValue};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
enum ModType {
    CharacterSkin,
    StageSkin,
    Core,
}

// TODO: Consider merging with `AffectedAsset`...
/// What "asset" the mod file is associated with (eg. skin slot 02).
#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub enum ModFileAssetAssociation {
    CharSkinSlot(CharSkinSlotValue),
    Stage(StageSlotValue),

    /// File is not specific to any slot but may affect multiple slots.
    Global,

    /// File is present in the mod but has no effect on the game and can be
    /// safely ignored.
    NoEffect,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VariantFileInfo {
    mod_type: Vec<ModType>,
    owned_files: HashMap<ModFileAssetAssociation, Vec<Utf8PathBuf>>,
}

impl VariantFileInfo {
    /// Crawl the mod directory looking for any files that we can classify.
    pub fn from_uncompressed_path(p: &Utf8Path) -> Self {
        todo!()
    }
}

#[derive(Debug)]
struct FileOwnerDb {
    mod_info: HashMap<ModId, VariantFileInfo>,
    files_with_associations: HashMap<Utf8PathBuf, ModId>,
}

#[cfg(test)]
mod tests {}
