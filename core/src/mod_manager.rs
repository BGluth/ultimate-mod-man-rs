use std::path::Path;

use thiserror::Error;
use ultimate_mod_man_rs_scraper::download_artifact_parser::SkinSlot;

use crate::{
    cmds::status::StatusCmdInfo,
    mod_db::{LoadPersistedStateErr, ModDb, ModIdentifier, ModWithVariantIdentifier},
};

pub type ModManagerResult<T> = Result<T, ModManagerErr>;

#[derive(Debug, Error)]
pub enum ModManagerErr {
    #[error(transparent)]
    LoadPersistedStateErr(#[from] LoadPersistedStateErr),
}

#[derive(Debug)]
pub struct ModManager {
    db: ModDb,
}

impl ModManager {
    pub fn new(mod_db_path: &Path) -> ModManagerResult<Self> {
        Ok(Self {
            db: ModDb::load_from_path(mod_db_path)?,
        })
    }

    pub fn status(&self, info: StatusCmdInfo) -> ModManagerResult<()> {
        todo!()
    }

    pub fn add_mods<I: IntoIterator<Item = ModIdentifier>>(
        &mut self,
        idents: I,
    ) -> ModManagerResult<()> {
        todo!()
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
        k: ModWithVariantIdentifier,
        s1: SkinSlot,
        s2: SkinSlot,
    ) -> ModManagerResult<()> {
        todo!()
    }

    pub fn switch_compare(&self) -> ModManagerResult<()> {
        todo!()
    }
}
