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

fn get_mod_page_for_mod_id(name: ModId) -> BananaScraperResult<String> {
    todo!()
}

pub fn resolve_mod_name(name: &str) -> BananaScraperResult<ModId> {
    todo!()
}

pub fn download_mod_variant(
    id: ModId,
    variant_name: &str,
) -> BananaScraperResult<ScrapedBananaModData> {
    todo!()
}
