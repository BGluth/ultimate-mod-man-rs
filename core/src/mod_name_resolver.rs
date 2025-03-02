use std::{collections::HashMap, fs};

use camino::Utf8Path;
use log::info;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ultimate_mod_man_rs_scraper::banana_scraper::{BananaClient, BananaScraperError};
use ultimate_mod_man_rs_utils::types::{ModId, ModIdentifier, VariantAndId, VariantAndIdentifier};

pub type ModNameResolverResult<T> = Result<T, ModNameResolverError>;

const CACHED_MOD_NAME_RESOLUTION_STATE_NAME: &str = "mod_name_resolution_cache.toml";

#[derive(Debug, Error)]
pub enum ModNameResolverError {
    #[error(transparent)]
    BananaScraperError(#[from] BananaScraperError),

    #[error(transparent)]
    ModResolutionDeserializationError(#[from] toml::de::Error),
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct BananaModNameResolver {
    local_cache: HashMap<String, ModId>,
}

impl BananaModNameResolver {
    pub(crate) fn new(p: &Utf8Path) -> ModNameResolverResult<Self> {
        let path = p.join(CACHED_MOD_NAME_RESOLUTION_STATE_NAME);

        let local_cache = match fs::read_to_string(path.clone()) {
            Ok(cache_data_str) => toml::from_str(&cache_data_str)?,
            Err(_) => {
                info!(
                    "No mod id resolution cache found at {:?}. Will create one during this run.",
                    path
                );
                HashMap::default()
            },
        };

        Ok(Self { local_cache })
    }

    pub(crate) async fn resolve_mod_ident(
        &mut self,
        scraper: &BananaClient,
        ident: &ModIdentifier,
    ) -> ModNameResolverResult<ModId> {
        match ident {
            ModIdentifier::Id(id) => Ok(*id),
            ModIdentifier::Name(name) => self.resolve_mod_name_to_id(scraper, name).await,
        }
    }

    async fn resolve_mod_name_to_id(
        &mut self,
        scraper: &BananaClient,
        name: &str,
    ) -> ModNameResolverResult<ModId> {
        let id = match self.local_cache.get(name) {
            Some(id) => *id,
            None => scraper.resolve_mod_name(name).await?,
        };

        Ok(id)
    }
}

impl BananaModNameResolver {
    pub(crate) async fn resolve_key(
        &mut self,
        key: VariantAndIdentifier,
        scraper: &BananaClient,
    ) -> ModNameResolverResult<VariantAndId> {
        Ok(VariantAndId {
            id: self.resolve_mod_ident(scraper, &key.ident).await?,
            variant_name: key.variant_name,
        })
    }
}
