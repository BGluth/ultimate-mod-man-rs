use std::{
    fs::{self, File},
    io::{self, Cursor, Read, Write},
    ops::Deref,
    path::PathBuf,
    str::FromStr,
};

use camino::{Utf8Path, Utf8PathBuf};
use log::warn;
use thiserror::Error;
use unrar::{Archive, error::UnrarError};
use zip::{ZipArchive, result::ZipError};

use crate::mod_file_classifier::CharSkinSlotValue;

const MAGIC_NUMBER_BYTE_READ_AMOUNT: usize = 100;

pub type VariantParseResult<T> = Result<T, VariantParseError>;
type ArchiveExpansionResult<T> = Result<T, ArchiveExpansionError>;

type TarError = io::Error;

#[derive(Debug, Error)]
pub enum VariantParseError {
    #[error("No magic number was present in the archive {0}")]
    NoMagicNumberInArchive(String),

    #[error("Unable to determine the archive type from it's magic number ({0})")]
    VariantPayloadNotARecognizableArchive(String, #[source] CompressionTypeFromExtStrErr),

    #[error(transparent)]
    InternArchiveParseError(#[from] InternArchiveParserErr),

    #[error(transparent)]
    ArchiveExpansionError(#[from] ArchiveExpansionError),

    #[error(transparent)]
    Io(#[from] TarError),
}

type InternArchiveParserResult<T> = Result<T, InternArchiveParserErr>;

#[derive(Debug, Error)]
pub enum ArchiveExpansionError {
    #[error(transparent)]
    InternArchiveParseError(#[from] InternArchiveParserErr),

    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum InternArchiveParserErr {
    #[error(transparent)]
    Zip(#[from] ZipError),

    #[error(transparent)]
    Rar(#[from] UnrarError),

    #[error(transparent)]
    Sevenz(#[from] sevenz_rust::Error),

    #[error(transparent)]
    Tar(#[from] io::Error),
}

pub struct ModPayloadParseInfo {
    expandable_archive: Box<dyn ExpandableArchive>,
}

impl ModPayloadParseInfo {
    pub fn new(archive_path: &Utf8Path) -> VariantParseResult<Self> {
        let variant_name = archive_path.file_name().unwrap();

        let archive_h = File::open(archive_path)?;
        let mut expandable_archive = Self::open_archive(variant_name, archive_path)?;

        let all_archive_file_paths = expandable_archive
            .get_paths_of_all_files()?
            .collect::<Vec<_>>();

        Ok(Self { expandable_archive })
    }

    fn open_archive(
        variant_file_name: &str,
        archive_path: &Utf8Path,
    ) -> VariantParseResult<Box<dyn ExpandableArchive>> {
        let mut header_bytes = Vec::with_capacity(MAGIC_NUMBER_BYTE_READ_AMOUNT);
        File::open(archive_path)?
            .take(MAGIC_NUMBER_BYTE_READ_AMOUNT as u64)
            .read_to_end(&mut header_bytes)?;

        let archive_format = Self::determine_archive_format(variant_file_name, &header_bytes)?;
        Self::open_archive_and_get_handle(archive_path, archive_format)
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
        if let Some(comp_type_from_ext) = Utf8PathBuf::from(variant_file_name).extension() {
            if let Ok(c_type_from_f_name) = CompressionType::from_str(comp_type_from_ext) {
                if c_type_from_f_name != magic_number_compression_type {
                    warn!(
                        "The magic number of the archive file does not match the extension in the file name. This is a bit weird, but regardless this is still fine."
                    );
                }
            }
        }

        Ok(magic_number_compression_type)
    }

    fn open_archive_and_get_handle<'a>(
        archive_path: &Utf8Path,
        comp_type: CompressionType,
    ) -> VariantParseResult<Box<dyn ExpandableArchive + 'a>> {
        let h: Box<dyn ExpandableArchive> = match comp_type {
            CompressionType::Zip => Box::new(ZipParser::new(fs::read(archive_path)?)?),
            CompressionType::Rar => Box::new(RarParser::new(archive_path.to_path_buf())?),
            CompressionType::SevenZip => Box::new(SevenZipParser::new(archive_path.to_path_buf())),
            CompressionType::Tar => Box::new(TarParser::new(archive_path)?),
        };

        Ok(h)
    }

    pub fn expand_archive_to_disk(self, dest_dir: &Utf8Path) -> VariantParseResult<()> {
        // Some mods (idk why) don't have the "root" mod directory at the very top, so we need to scan before decompression and look for it.
        let mod_root_directory_offset = self.search_for_mod_root();
        let root_offset = mod_root_directory_offset.as_ref().map(|x| x.as_path());
        let f = Box::new(Self::filter_fn);

        self.expandable_archive
            .expand_archive_to_disk_with_filter_and_offset(dest_dir, root_offset, f)?;

        Ok(())
    }

    fn filter_fn(p: &Utf8Path) -> bool {
        todo!()
    }

    fn remove_root_offset_from_path(p: &Utf8Path, root_offset_p: &Utf8Path) -> Utf8PathBuf {
        todo!()
    }

    fn search_for_mod_root(&self) -> Option<Utf8PathBuf> {
        todo!()
    }
}

pub trait ExpandableFile {
    fn write(&mut self, path: &Utf8Path) -> VariantParseResult<()> {
        todo!()
    }
}

pub trait ExpandableArchive {
    fn expand_archive_to_disk_with_filter_and_offset(
        &self,
        dest_dir: &Utf8Path,
        root_offset: Option<&Utf8Path>,
        filter: Box<dyn Fn(&Utf8Path) -> bool>,
    ) -> ArchiveExpansionResult<()>;

    fn get_paths_of_all_files<'a>(
        &'a mut self,
    ) -> ArchiveExpansionResult<Box<dyn Iterator<Item = Utf8PathBuf> + 'a>>;
}

#[derive(Debug)]
struct ZipParser {
    intern: ZipArchive<Cursor<Vec<u8>>>,
}

impl ExpandableArchive for ZipParser {
    fn expand_archive_to_disk_with_filter_and_offset(
        &self,
        dest_dir: &Utf8Path,
        root_offset: Option<&Utf8Path>,
        filter: Box<dyn Fn(&Utf8Path) -> bool>,
    ) -> ArchiveExpansionResult<()> {
        todo!()
    }

    fn get_paths_of_all_files(
        &mut self,
    ) -> ArchiveExpansionResult<Box<dyn Iterator<Item = Utf8PathBuf>>> {
        todo!()
    }
}

impl ZipParser {
    fn new(compressed_bytes: Vec<u8>) -> InternArchiveParserResult<Self> {
        let intern = ZipArchive::new(Cursor::new(compressed_bytes))?;

        Ok(Self { intern })
    }
}

struct RarParser {
    archive_path: Utf8PathBuf,
    all_file_paths: Vec<PathBuf>,
}

impl ExpandableArchive for RarParser {
    fn expand_archive_to_disk_with_filter_and_offset(
        &self,
        dest_dir: &Utf8Path,
        root_offset: Option<&Utf8Path>,
        filter: Box<dyn Fn(&Utf8Path) -> bool>,
    ) -> ArchiveExpansionResult<()> {
        todo!()
    }

    fn get_paths_of_all_files(
        &mut self,
    ) -> ArchiveExpansionResult<Box<dyn Iterator<Item = Utf8PathBuf>>> {
        todo!()
    }
}

impl RarParser {
    fn new(archive_path: Utf8PathBuf) -> InternArchiveParserResult<Self> {
        let h = Archive::new(&archive_path).open_for_listing()?;

        let mut all_file_paths = Vec::new();
        for header_res in h {
            let header = header_res?;
            all_file_paths.push(header.filename);
        }

        let res = RarParser {
            archive_path,
            all_file_paths,
        };

        Ok(res)
    }
}

#[derive(Debug)]
struct SevenZipParser {
    path: Utf8PathBuf,
}

impl SevenZipParser {
    fn new(path: Utf8PathBuf) -> Self {
        Self { path }
    }
}

impl ExpandableArchive for SevenZipParser {
    fn expand_archive_to_disk_with_filter_and_offset(
        &self,
        dest_dir: &Utf8Path,
        root_offset: Option<&Utf8Path>,
        filter: Box<dyn Fn(&Utf8Path) -> bool>,
    ) -> ArchiveExpansionResult<()> {
        todo!()
    }

    fn get_paths_of_all_files(
        &mut self,
    ) -> ArchiveExpansionResult<Box<dyn Iterator<Item = Utf8PathBuf>>> {
        let h = sevenz_rust::Archive::open(&self.path).map_err(InternArchiveParserErr::from)?;
        Ok(Box::new(
            h.files
                .into_iter()
                .filter(|f| !f.is_directory())
                .map(|f| f.name.into()),
        ))
    }
}

struct TarParser {
    intern: tar::Archive<File>,
}

impl TarParser {
    fn new(path: &Utf8Path) -> Result<Self, TarError> {
        let intern = tar::Archive::new(File::open(path)?);

        Ok(Self { intern })
    }
}

impl ExpandableArchive for TarParser {
    fn expand_archive_to_disk_with_filter_and_offset(
        &self,
        dest_dir: &Utf8Path,
        root_offset: Option<&Utf8Path>,
        filter: Box<dyn Fn(&Utf8Path) -> bool>,
    ) -> ArchiveExpansionResult<()> {
        todo!()
    }

    fn get_paths_of_all_files<'a>(
        &'a mut self,
    ) -> ArchiveExpansionResult<Box<dyn Iterator<Item = Utf8PathBuf> + 'a>> {
        // Going to do a vec allocation for the sake of maintaining a nicer interface.
        Ok(Box::new(self.intern.entries()?.collect::<Result<Vec<_>, _>>()?.into_iter()
            .map(|f| Utf8Path::from_path(f.path().expect("Failed to get the path of a item in a tar file! (Are you running on Windows and unpacking a tar file that is not unicode?")
            .deref()).unwrap().to_path_buf())))
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
pub struct CompressionTypeFromExtStrErr(String);

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
    mod_default_slots: Vec<CharSkinSlotValue>,
}
