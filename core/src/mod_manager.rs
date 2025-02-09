use camino::Utf8Path;
use log::info;
use thiserror::Error;
use ultimate_mod_man_rs_scraper::{
    banana_scraper::{BananaClient, BananaScraperError},
    download_artifact_parser::SkinSlot,
};
use ultimate_mod_man_rs_utils::{
    types::{ModIdentifier, VariantAndIdentifier},
    user_input_delegate::UserInputDelegate,
};

use crate::{
    cmds::status::StatusCmdInfo,
    mod_db::{LoadPersistedStateErr, ModDb, ModDbError},
    mod_name_resolver::{BananaModNameResolver, ModNameResolverError},
};

pub type ModManagerResult<T> = Result<T, ModManagerErr>;

#[derive(Debug, Error)]
pub enum ModManagerErr {
    #[error(transparent)]
    LoadPersistedStateErr(#[from] LoadPersistedStateErr),

    #[error(transparent)]
    ModDbError(#[from] ModDbError),

    #[error(transparent)]
    BananaScraperError(#[from] BananaScraperError),

    #[error(transparent)]
    ModNameResolverError(#[from] ModNameResolverError),
}

#[derive(Debug)]
pub struct ModManager<U: UserInputDelegate> {
    db: ModDb,
    scraper: BananaClient,
    mod_resolution_cache: BananaModNameResolver,
    user_input_delegate: U,
}

impl<U: UserInputDelegate> ModManager<U> {
    pub fn new(mod_db_path: &Utf8Path, user_input_delegate: U) -> ModManagerResult<Self> {
        Ok(Self {
            db: ModDb::load_from_path(mod_db_path)?,
            scraper: BananaClient::new()?,
            mod_resolution_cache: BananaModNameResolver::new(mod_db_path)?,
            user_input_delegate,
        })
    }

    pub fn status(&self, info: StatusCmdInfo) -> ModManagerResult<()> {
        todo!()
    }

    pub async fn add_mods<I: IntoIterator<Item = VariantAndIdentifier>>(
        &mut self,
        idents: I,
    ) -> ModManagerResult<()> {
        for ident_and_variant in idents {
            let id_and_variant = self
                .mod_resolution_cache
                .resolve_key(ident_and_variant.clone(), &self.scraper)
                .await?;

            if self.db.exists(&id_and_variant) {
                info!(
                    "Skipping adding the mod variant {} since it was already installed. (If you want to check for mod updates, run the update command.)",
                    ident_and_variant
                );
                return Ok(());
            }

            // Mod is not installed.
            let downloaded_mod_variant = self
                .scraper
                .download_mod_variant(&mut self.user_input_delegate, &id_and_variant)
                .await?;

            self.db.add(&id_and_variant, downloaded_mod_variant)?;
        }

        Ok(())
    }

    pub fn delete_mods<I: IntoIterator<Item = ModIdentifier>>(
        &mut self,
        idents: I,
    ) -> ModManagerResult<()> {
        todo!()
    }

    pub fn sync_with_switch() -> ModManagerResult<()> {
        todo!()
    }

    pub fn check_for_updates(&mut self) -> ModManagerResult<()> {
        todo!()
    }

    pub fn enable_disable<I: IntoIterator<Item = ModIdentifier>>(
        &mut self,
        enable: bool,
    ) -> ModManagerResult<()> {
        todo!()
    }

    pub fn resolve_conflicts(&mut self) -> ModManagerResult<()> {
        todo!()
    }

    pub fn change_slot(
        &mut self,
        k: VariantAndIdentifier,
        s1: SkinSlot,
        s2: SkinSlot,
    ) -> ModManagerResult<()> {
        todo!()
    }

    pub fn switch_compare(&self) -> ModManagerResult<()> {
        todo!()
    }
}
