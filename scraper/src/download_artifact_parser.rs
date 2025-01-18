use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug)]
pub(crate) struct VariantSkinParseInfo {}

// TODO: Work out how to support more than just skins...
#[derive(Debug)]
pub(crate) struct CharacterSlot {
    /// The name of the character. Keeping this dynamic in order to support custom characters and not just skins.
    char_name: String,

    /// The slots specified by the mod.
    mod_default_slots: Vec<SkinSlot>,
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
