use thiserror::Error;

use crate::types::ModId;

pub type BananaScraperResult<T> = Result<T, BananaScraperError>;

#[derive(Debug, Error)]
pub enum BananaScraperError {}

#[derive(Debug)]
pub(crate) struct ScrapedBananaModData {
    name: String,
    version: Option<String>,
    variant_download_artifact: Vec<u8>,
}

pub(crate) fn download_mod_variant(
    id: ModId,
    variant_name: String,
) -> BananaScraperResult<ScrapedBananaModData> {
    todo!()
}
