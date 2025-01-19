use thiserror::Error;

use crate::types::ModId;

pub type BananaScraperResult<T> = Result<T, BananaScraperError>;

#[derive(Debug, Error)]
pub enum BananaScraperError {}

#[derive(Debug)]
pub struct ScrapedBananaModData {
    name: String,
    version: Option<String>,
    variant_download_artifact: Vec<u8>,
}

pub fn get_mod_id_for_name(name: &str) -> ModId {
    todo!()
}

pub fn download_mod_variant(
    id: ModId,
    variant_name: String,
) -> BananaScraperResult<ScrapedBananaModData> {
    todo!()
}
