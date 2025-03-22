use std::{
    convert::Infallible,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use camino::Utf8PathBuf;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type ModId = u64;
pub type SkinSlotIdx = usize;
pub type StageSlotIdx = usize;

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
        // If we can parse it as a ID (`u64`), then treat it as an ID. Otherwise just
        // assume that we received the mod name.``
        s.parse::<u64>()
            .map(ModIdentifier::Id)
            .or_else(|_| Ok(ModIdentifier::Name(s.to_string())))
    }
}

impl Display for ModIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ModIdentifier::Id(id) => write!(f, "{}", id),
            ModIdentifier::Name(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Error)]
pub enum VariantAndIdentifierStrError {
    #[error("Missing \"/\" when parsing the mod variant and identifier string \"{0}\"")]
    MissingSlashSeparator(String),

    #[error(
        "Missing \"/\" variant (left hand side of \"/\" when parsing the mod variant and \
         identifier string \"{0}\""
    )]
    MissingVariant(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VariantAndId {
    pub id: ModId,
    pub variant_name: String,
}

impl Display for VariantAndId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.id, self.variant_name)
    }
}

impl VariantAndId {
    pub fn new(id: ModId, variant_name: String) -> Self {
        Self { id, variant_name }
    }
}

/// Because mods can have multiple install payloads that affect multiple slots,
/// we need a key type that identifies a mod and one of its specific variants.
#[derive(Builder, Clone, Debug, Eq, PartialEq, Hash)]
pub struct VariantAndIdentifier {
    #[builder(setter(custom = true))]
    pub ident: ModIdentifier,
    pub variant_name: String,
}

impl FromStr for VariantAndIdentifier {
    type Err = VariantAndIdentifierStrError;

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
                return Err(VariantAndIdentifierStrError::MissingSlashSeparator(
                    s.to_string(),
                ));
            },
            Some(s) if s.is_empty() => {
                return Err(VariantAndIdentifierStrError::MissingVariant(s.to_string()));
            },
            Some(s) => s.to_string(),
        };

        Ok(Self {
            ident,
            variant_name,
        })
    }
}

impl Display for VariantAndIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.ident, self.variant_name)
    }
}

impl VariantAndIdentifierBuilder {
    pub fn ident_id(&mut self, id: ModId) -> &mut Self {
        self.ident = Some(id.into());
        self
    }

    pub fn ident_name(&mut self, name: String) -> &mut Self {
        self.ident = Some(name.into());
        self
    }
}

#[derive(Clone, Debug)]
pub enum AssetSlot {
    CharacterSkin(CharSkinSlotValue),
    StageSkin(StageSlotValue),
    Global(Utf8PathBuf),
}

#[derive(Debug)]
pub enum AvailableSlotsToSwapToInfo {
    CharacterSkin(Vec<SkinSlotValue>),
    // TODO: Add music...
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct CharSkinSlotValue {
    char_key: String,
    skin_slot_idx: SkinSlotValue,
}

impl Display for CharSkinSlotValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.char_key, self.skin_slot_idx)
    }
}

/// A skin slot can actually go beyond 0 - 7 where anything beyond `7` is used
/// for special purposes. We still need to detect collisions in these ranges,
/// although maybe we can have less strict logic for handling them.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SkinSlotValue(u8);

impl Display for SkinSlotValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            0..=7 => write!(f, "Skin slot")?,
            _ => write!(f, "Custom skin slot")?,
        };

        write!(f, " C{:2x} ({})", self.0, self.0)
    }
}

impl SkinSlotValue {
    pub fn is_normal_skin_slot(&self) -> bool {
        matches!(self.0, 0..=7)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct StageSlotValue(u8);

#[derive(Debug)]
pub enum PickedResolutionOption {
    NonSwapOption(PickedNonSwappableResolutionOption),
    Swap(usize),
}

#[derive(Debug)]
pub enum PickedNonSwappableResolutionOption {
    KeepExisting,
    Replace,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{ModId, VariantAndIdentifier, VariantAndIdentifierBuilder};
    use crate::types::{ModIdentifier, VariantAndIdentifierStrError};

    #[test]
    fn mod_identifier_from_name_string_works() {
        assert_eq!(ModIdentifier::from_str("./rust_mod").unwrap(), "./rust_mod");
    }

    #[test]
    fn mod_identifier_from_id_string_works() {
        assert_eq!(ModIdentifier::from_str("9001").unwrap(), 9001)
    }

    fn m_var_ident_from_str(str: &str) -> VariantAndIdentifier {
        VariantAndIdentifier::from_str(str).unwrap()
    }

    fn m_var_ident_test_payload_from_id(id: ModId, var_name: &str) -> VariantAndIdentifier {
        VariantAndIdentifierBuilder::default()
            .ident_id(id)
            .variant_name(var_name.to_string())
            .build()
            .unwrap()
    }

    fn m_var_ident_test_payload_from_name(name: &str, var_name: &str) -> VariantAndIdentifier {
        VariantAndIdentifierBuilder::default()
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
        println!("{:?}", VariantAndIdentifier::from_str(""));

        assert!(matches!(
            VariantAndIdentifier::from_str(""),
            Err(VariantAndIdentifierStrError::MissingSlashSeparator(_))
        ));

        assert!(matches!(
            VariantAndIdentifier::from_str("just_a_mod_nameVariantAndIdentifier::from_str"),
            Err(VariantAndIdentifierStrError::MissingSlashSeparator(_))
        ));

        assert!(matches!(
            VariantAndIdentifier::from_str("just_a_mod_name/"),
            Err(VariantAndIdentifierStrError::MissingVariant(_))
        ));
    }
}
