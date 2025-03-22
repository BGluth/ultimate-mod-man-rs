use std::ops::Deref;

use camino::Utf8Path;
use log::info;
use thiserror::Error;
use ultimate_mod_man_rs_scraper::banana_scraper::{BananaClient, BananaScraperError};
use ultimate_mod_man_rs_utils::{
    types::{
        ModIdentifier, PickedResolutionOption, SkinSlotValue, VariantAndId, VariantAndIdentifier,
    },
    user_input_delegate::UserInputDelegate,
};

use crate::{
    cmds::status::StatusCmdInfo,
    in_prog_action::{Action, InProgAction},
    mod_db::{ConflictingModVariant, ModDb, ModDbError, UnableToEnableReason},
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
    pub fn new(cache_dir_path: &Utf8Path, user_input_delegate: U) -> ModManagerResult<Self> {
        Ok(Self {
            db: ModDb::load_from_path(cache_dir_path)?,
            scraper: BananaClient::new()?,
            mod_resolution_cache: BananaModNameResolver::new(cache_dir_path)?,
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

            if self.db.exists(&key) {
                info!(
                    "Skipping adding the mod variant {} since it was already installed. (If you \
                     want to check for mod updates, run the update command.)",
                    ident_and_variant
                );
                return Ok(());
            }

            if let Some(reason) = self.db.enable_check(&key) {
                match reason {
                    UnableToEnableReason::Conflicts(conflicts) => {
                        info!("Conflicts detected when trying to enable {}!", key);
                        self.handle_variant_add_conflicts(&key, &conflicts);
                    },
                    UnableToEnableReason::AlreadyEnabled => unreachable!(),
                }
            }

            self.db
                .journal_action_as_in_prog(Action::Add(key.clone()))?;

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

    fn handle_variant_add_conflicts(
        &mut self,
        key: &VariantAndId,
        variant_conflicts: &[ConflictingModVariant],
    ) {
        let summary = self
            .db
            .get_variant_conflict_summary(key)
            .unwrap_or_else(|| {
                panic!(
                    "Expected to find conflicts for mod variant {} but none were found! This is a \
                     bug!",
                    key
                )
            });
        self.user_input_delegate
            .display_variant_conflict_summary(&summary);

        // The mod that we want to enable has one or more conflicts with other mods.
        for variant_conflict in variant_conflicts {
            // Start a "transaction" of configuring mods that takes effect once all
            // conflicts are resolved.
            let mut mod_db_txn = self.db.resolve_conflict(variant_conflict);

            while let Some(sub_conflict) = mod_db_txn.get_next_conflict_to_resolve() {
                let slot_conflict = sub_conflict.slot();

                let picked_res = match sub_conflict.swappable_info() {
                    Some(available_slots) => {
                        // We can swap this.
                        self.user_input_delegate
                            .get_variant_conflict_resolution_option_swappable(
                                &variant_conflict.key,
                                key,
                                slot_conflict,
                                &available_slots,
                            )
                    },
                    None => {
                        // Not swappable.
                        let res = self
                            .user_input_delegate
                            .get_variant_conflict_resolution_option_non_swappable(
                                &variant_conflict.key,
                                key,
                                slot_conflict,
                            );
                        PickedResolutionOption::NonSwapOption(res)
                    },
                };

                mod_db_txn.resolve_conflict(picked_res);
            }

            // All conflicts have been resolved. Commit the changes to the DB.
            mod_db_txn.commit();
        }
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
        char_key: &str,
        s1: SkinSlotValue,
        s2: SkinSlotValue,
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
            },
        }

        // We finished cleaning up the in progress action, so now we can remove it from
        // disk.
        self.db.remove_in_prog_action()?;

        Ok(())
    }
}
