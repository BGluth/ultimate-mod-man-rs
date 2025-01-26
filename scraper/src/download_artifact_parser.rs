use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use log::warn;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type VariantParseResult<T> = Result<T, VariantParseError>;

#[derive(Debug, Error)]
pub enum VariantParseError {
    #[error("No magic number was present in the archive {0}")]
    NoMagicNumberInArchive(String),

    #[error("Unable to determine the archive type from it's magic number ({0})")]
    VariantPayloadNotARecognizableArchive(String, #[source] CompressionTypeFromExtStrErr),
}

pub struct ModPayloadParseInfo {
    compressed_bytes: Vec<u8>,

    /// Some mods (idk why) don't have the "root" mod directory at the very top, so we need to scan before decompression and look for it.
    mod_root_directory_offset: Option<PathBuf>,

    expandable_archive: Box<dyn ExpandableArchive>,
}

impl ModPayloadParseInfo {
    pub fn new(variant_name: &str, compressed_bytes: Vec<u8>) -> VariantParseResult<Self> {
        let expandable_archive = Self::open_archive(variant_name, &compressed_bytes)?;

        let all_archive_file_paths = expandable_archive
            .get_paths_of_all_files()
            .collect::<Vec<_>>();
        let mod_root_directory_offset = Self::search_for_mod_root(&expandable_archive);

        Ok(Self {
            compressed_bytes,
            mod_root_directory_offset,
            expandable_archive,
        })
    }

    fn open_archive(
        variant_file_name: &str,
        compressed_bytes: &[u8],
    ) -> VariantParseResult<Box<dyn ExpandableArchive>> {
        let archive_format = Self::determine_archive_format(variant_file_name, compressed_bytes)?;

        todo!()
    }

    fn determine_archive_format(
        variant_file_name: &str,
        compressed_bytes: &[u8],
    ) -> VariantParseResult<CompressionType> {
        let type_from_magic_number = infer::get(compressed_bytes);

        // We're always going to determine the type of the archive by the magic number, but will output a warning if the file extension does not match.
        let archive_type_str = type_from_magic_number
            .ok_or_else(|| {
                VariantParseError::NoMagicNumberInArchive(variant_file_name.to_string())
            })?
            .extension();

        let magic_number_compression_type =
            CompressionType::from_str(archive_type_str).map_err(|err| {
                VariantParseError::VariantPayloadNotARecognizableArchive(
                    variant_file_name.to_string(),
                    err,
                )
            })?;

        // Just as an additional check, if the file has a visible extension that we recognize, if it differs from the magic number, output a warning to the user (but continue going).
        if let Some(comp_type_from_ext) = PathBuf::from(variant_file_name).extension() {
            if let Ok(c_type_from_f_name) =
                CompressionType::from_str(comp_type_from_ext.to_string_lossy().as_ref())
            {
                if c_type_from_f_name != magic_number_compression_type {
                    warn!(
                        "The magic number of the archive file does not match the extension in the file name. This is a bit weird, but regardless this is still fine."
                    );
                }
            }
        }

        Ok(magic_number_compression_type)
    }

    fn open_archive_and_get_handle(
        compressed_bytes: &[u8],
        comp_type: CompressionType,
    ) -> VariantParseResult<Box<dyn ExpandableArchive>> {
        let h = match comp_type {
            CompressionType::Zip => todo!(),
            CompressionType::Rar => todo!(),
            CompressionType::SevenZip => todo!(),
            CompressionType::Tar => todo!(),
        };

        Ok(h)
    }

    fn search_for_mod_root(archive: &Box<dyn ExpandableArchive>) -> Option<PathBuf> {
        todo!()
    }
}

pub trait ExpandableFile {
    fn write(&mut self, path: &Path) -> VariantParseResult<()> {
        todo!()
    }
}

pub trait ExpandableArchive {
    fn get_file_names_and_write_handles(
        &self,
    ) -> Box<dyn Iterator<Item = (PathBuf, Box<dyn ExpandableFile>)>>;
    fn get_paths_of_all_files(&self) -> Box<dyn Iterator<Item = PathBuf>>;
}

#[derive(Debug)]
struct ZipParser {}

impl ExpandableArchive for ZipParser {
    fn get_file_names_and_write_handles(
        &self,
    ) -> Box<dyn Iterator<Item = (PathBuf, Box<dyn ExpandableFile>)>> {
        todo!()
    }

    fn get_paths_of_all_files(&self) -> Box<dyn Iterator<Item = PathBuf>> {
        todo!()
    }
}

impl ZipParser {
    fn new(compression_bytes: Vec<u8>) -> Self {
        todo!()
    }
}

#[derive(Debug)]
struct RarParser {}

impl ExpandableArchive for RarParser {
    fn get_file_names_and_write_handles(
        &self,
    ) -> Box<dyn Iterator<Item = (PathBuf, Box<dyn ExpandableFile>)>> {
        todo!()
    }

    fn get_paths_of_all_files(&self) -> Box<dyn Iterator<Item = PathBuf>> {
        todo!()
    }
}

#[derive(Debug)]
struct SevenZipParser {}

impl ExpandableArchive for SevenZipParser {
    fn get_file_names_and_write_handles(
        &self,
    ) -> Box<dyn Iterator<Item = (PathBuf, Box<dyn ExpandableFile>)>> {
        todo!()
    }

    fn get_paths_of_all_files(&self) -> Box<dyn Iterator<Item = PathBuf>> {
        todo!()
    }
}

#[derive(Debug)]
struct TarParser {}

impl ExpandableArchive for TarParser {
    fn get_file_names_and_write_handles(
        &self,
    ) -> Box<dyn Iterator<Item = (PathBuf, Box<dyn ExpandableFile>)>> {
        todo!()
    }

    fn get_paths_of_all_files(&self) -> Box<dyn Iterator<Item = PathBuf>> {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompressionType {
    Zip,
    Rar,
    SevenZip,
    Tar,
}

#[derive(Debug, Error)]
#[error("Unknown compressed archive type for extension {0}")]
struct CompressionTypeFromExtStrErr(String);

impl FromStr for CompressionType {
    type Err = CompressionTypeFromExtStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zip" => Ok(Self::Zip),
            "rar" => Ok(Self::Rar),
            "7z" => Ok(Self::SevenZip),
            "tar" | "xz" => Ok(Self::Tar),
            _ => Err(CompressionTypeFromExtStrErr(s.to_string())),
        }
    }
}

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
