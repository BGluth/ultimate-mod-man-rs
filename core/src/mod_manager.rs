use std::ops::Deref;

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
    in_prog_action::{Action, InProgAction},
    mod_db::{ModDb, ModDbError},
    mod_name_resolver::{BananaModNameResolver, ModNameResolverError},
};

pub type ModManagerResult<T> = Result<T, ModManagerErr>;

#[derive(Debug, Error)]
pub enum ModManagerErr {
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
        self.cleanup_any_incomplete_in_prog_action()?;

        for ident_and_variant in idents {
            let key = self
                .mod_resolution_cache
                .resolve_key(ident_and_variant.clone(), &self.scraper)
                .await?;

            self.db
                .journal_action_as_in_prog(Action::Add(key.clone()))?;

            if self.db.exists(&key) {
                info!(
                    "Skipping adding the mod variant {} since it was already installed. (If you want to check for mod updates, run the update command.)",
                    ident_and_variant
                );
                return Ok(());
            }

            // Mod is not installed.
            let downloaded_mod_variant = self
                .scraper
                .download_mod_variant(&mut self.user_input_delegate, &key)
                .await?;

            self.db.add(&key, downloaded_mod_variant)?;

            self.db.remove_in_prog_action()?;
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

    fn cleanup_any_incomplete_in_prog_action(&mut self) -> ModManagerResult<()> {
        if let Some(in_prog_act) = self.db.get_in_prog_action_if_any()? {
            self.remove_state_of_incomplete_in_prog_action(in_prog_act)?;
        }

        Ok(())
    }

    fn remove_state_of_incomplete_in_prog_action(
        &mut self,
        action: InProgAction,
    ) -> ModManagerResult<()> {
        match action.deref() {
            Action::Add(key) => {
                self.db.remove_variant(key)?;
            }
        }

        // We finished cleaning up the in progress action, so now we can remove it from disk.
        self.db.remove_in_prog_action()?;

        Ok(())
    }
}
