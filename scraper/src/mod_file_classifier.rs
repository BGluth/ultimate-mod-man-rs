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

#[derive(Debug)]
enum ModType {
    CharacterSkin,
    StageSkin,
    Core,
}

#[derive(Debug)]
enum OwnerType {
    SkinSlot(SkinSlotValue),
    Stage(StageSlotValue),
}

#[derive(Debug)]
pub(crate) struct ClassifiedModInfo {
    mod_type: ModType,
    owned_files: HashMap<OwnerType, Vec<Utf8PathBuf>>,
}

impl ClassifiedModInfo {
    /// Crawl the mod directory looking for any files that we can classify.
    pub(crate) fn from_uncompressed_path(p: &Utf8Path) -> Self {
        todo!()
    }
}

#[derive(Debug)]
struct FileOwnerDb {
    mod_info: HashMap<ModId, ClassifiedModInfo>,
    files_with_associations: HashMap<Utf8PathBuf, ModId>,
}

/// A skin slot can actually go beyond 0 - 7 where anything beyond `7` is used for special purposes. We still need to detect collisions in these ranges, although maybe we can have less strict logic for handling them.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SkinSlotValue(u8);

impl SkinSlotValue {
    fn is_normal_skin_slot(&self) -> bool {
        matches!(self.0, 0..=7)
    }
}

impl Display for SkinSlotValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            0..=7 => write!(f, "Skin slot")?,
            _ => write!(f, "Custom skin slot")?,
        };

        write!(f, " C{:2x} ({})", self.0, self.0)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct StageSlotValue(u8);

#[cfg(test)]
mod tests {}
