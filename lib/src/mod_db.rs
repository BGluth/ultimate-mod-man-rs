use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type ModId = u64;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ModDb {
    entries: Vec<ModEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ModEntry {
    /// The ID of the mod on GameBanana.
    id: ModId,

    /// The name of the mod on GameBanana.
    name: String,

    /// Because there can be different variants available to download for a given mod, we need to also be able to specify which one we are using.
    download_link_name: String,

    /// Slots used by the skin and any overrides.
    slots_and_overrides: CharacterSlotsAndOverrides,

    /// The version that we have in the mod manager.
    version: ModVersion,

    /// Whether or not the mod is enabled.
    enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CharacterSlotsAndOverrides {
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
pub(crate) enum SkinSlot {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ModVersion {}
