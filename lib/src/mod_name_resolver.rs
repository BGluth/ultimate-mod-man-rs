use std::path::Path;

use thiserror::Error;

use crate::mod_db::{ModId, ModIdentifier};

pub type ModNameResolverResult<T> = Result<T, ModNameResolverError>;

#[derive(Debug, Error)]
pub enum ModNameResolverError {}

#[derive(Debug)]
pub(crate) struct BananaModNameResolver {
    local_cache: LocalBananaNameResolveCache,
}

impl BananaModNameResolver {
    pub(crate) fn new(p: &Path) -> Self {
        todo!()
    }

    pub(crate) fn resolve_mod_ident(&mut self, ident: ModIdentifier) -> ModId {
        todo!()
    }

    fn resolve_mod_name_to_id(&mut self, name: &str) -> ModId {
        todo!()
    }
}

#[derive(Debug)]
struct LocalBananaNameResolveCache {}

impl LocalBananaNameResolveCache {
    fn load_from_path(p: &Path) -> Self {
        todo!()
    }
}
