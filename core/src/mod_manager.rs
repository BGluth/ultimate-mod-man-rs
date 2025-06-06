use std::ops::Deref;

use camino::Utf8Path;
use log::{info, warn};
use thiserror::Error;
use ultimate_mod_man_rs_scraper::banana_scraper::{BananaClient, BananaScraperError};
use ultimate_mod_man_rs_utils::{
    types::{
        AssetSlot, AvailableSlotsToSwapToInfo, PickedResolutionOption, SkinSlotValue,
        SwappableAssetSlot, VariantAndId, VariantAndIdentifier,
    },
    user_input_delegate::{SlotInfo, UserInputDelegate},
};

use crate::{
    cmds::status::StatusCmdInfo,
    in_prog_action::{Action, InProgAction},
    mod_db::{
        AssetConflict, ModDb, ModDbError, SwappableAssetConflict, UnableToEnableReason,
        VariantConflictInfo,
    },
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

            self.db
                .journal_action_as_in_prog(Action::Add(key.clone()))?;

            // Mod is not installed.
            let downloaded_mod_variant = self
                .scraper
                .download_mod_variant(&mut self.user_input_delegate, &key)
                .await?;

            self.db.add_variant(&key, downloaded_mod_variant)?;

            if let Some(reason) = self.db.enable_variant(&key)? {
                match reason {
                    UnableToEnableReason::Conflicts(conflicts) => {
                        info!("Conflicts detected when trying to enable {}!", key);
                        self.handle_variant_add_conflicts(&key, &conflicts);
                    },
                    UnableToEnableReason::AlreadyEnabled => unreachable!(),
                }
            }

            self.db.remove_in_prog_action()?;
        }

        Ok(())
    }

    fn handle_variant_add_conflicts(
        &mut self,
        key: &VariantAndId,
        variant_conflicts: &VariantConflictInfo,
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
        for variant_conflict in variant_conflicts.conflicts.iter() {
            // Start a "transaction" of configuring mods that takes effect once all
            // conflicts are resolved.
            let mut mod_db_txn = self.db.resolve_conflict(variant_conflict);

            while let Some(sub_conflict) = mod_db_txn.get_next_conflict_to_resolve() {
                // let slot_conflict = sub_conflict.slot();

                let swappable_conflict = match sub_conflict {
                    // Asset is swappable.
                    AssetConflict::Swappable(info) => {
                        // We can swap this.
                        let swappable_slot = info.existing();
                        let available_slots = info.slots_available_to_swap_into();

                        self.user_input_delegate
                            .get_variant_conflict_resolution_option_swappable(
                                &variant_conflict.key,
                                key,
                                &info.into(),
                                available_slots.num_slot_open(),
                            )
                    },
                    AssetConflict::NonSwappable(info) => {
                        // Not swappable.
                        let res = self
                            .user_input_delegate
                            .get_variant_conflict_resolution_option_non_swappable(
                                &variant_conflict.key,
                                key,
                                todo!(),
                            );
                        PickedResolutionOption::NonSwapOption(res)
                    },
                };

                mod_db_txn.resolve_conflict(swappable_conflict);
            }

            // All conflicts have been resolved. Commit the changes to the DB.
            mod_db_txn.commit();
        }
    }

    pub async fn delete_variants<I: IntoIterator<Item = VariantAndIdentifier>>(
        &mut self,
        idents: I,
    ) -> ModManagerResult<()> {
        self.cleanup_any_incomplete_in_prog_action()?;

        for ident in idents {
            let key = self
                .mod_resolution_cache
                .resolve_key(ident.clone(), &self.scraper)
                .await?;

            if !self.db.exists(&key) {
                info!(
                    "Skipping deleting the mod variant {} since it was not installed.",
                    ident
                );
                return Ok(());
            }

            self.db
                .journal_action_as_in_prog(Action::Remove(key.clone()))?;
            self.db.remove_variant(&key)?;
            self.db.remove_in_prog_action()?;
        }

        Ok(())
    }

    pub fn sync_with_switch() -> ModManagerResult<()> {
        todo!()
    }

    pub fn check_for_updates(&mut self) -> ModManagerResult<()> {
        todo!()
    }

    pub async fn enable_disable<I: IntoIterator<Item = VariantAndIdentifier>>(
        &mut self,
        idents: I,
        enable: bool,
    ) -> ModManagerResult<()> {
        for ident in idents {
            let key = self
                .mod_resolution_cache
                .resolve_key(ident.clone(), &self.scraper)
                .await?;

            if !self.db.exists(&key) {
                info!(
                    "Skipping enabling the mod variant {} since it was not installed.",
                    key
                );
                continue;
            }

            match enable {
                false => {
                    self.db.disable_variant(key);
                },
                true => {
                    if let Some(reason) = self.db.enable_variant(&key)? {
                        match reason {
                            UnableToEnableReason::Conflicts(variant_conflicts) => {
                                self.handle_variant_add_conflicts(&key, &variant_conflicts);
                            },
                            UnableToEnableReason::AlreadyEnabled => {
                                info!(
                                    "Skipping enabling the mod {} because it was already enabled.",
                                    key
                                );
                                continue;
                            },
                        }
                    }
                },
            }
        }

        Ok(())
    }

    pub async fn change_slot(
        &mut self,
        ident: VariantAndIdentifier,
        slot: SwappableAssetSlot,
    ) -> ModManagerResult<()> {
        let key = self
            .mod_resolution_cache
            .resolve_key(ident, &self.scraper)
            .await?;

        if !self.db.exists(&key) {
            warn!("Mod variant {} does not exist!", key);
        }

        let available_slots = self.db.get_available_slots_to_swap_to(&slot);
        self.user_input_delegate
            .choose_slot_to_swap_to(slot, &available_slots);

        Ok(())
    }

    pub fn switch_compare(&self) -> ModManagerResult<()> {
        todo!()
    }

    fn cleanup_any_incomplete_in_prog_action(&mut self) -> ModManagerResult<()> {
        if let Some(in_prog_act) = self.db.get_in_prog_action_if_any()? {
            self.handle_incomplete_in_prog_action(in_prog_act)?;
        }

        Ok(())
    }

    fn handle_incomplete_in_prog_action(&mut self, action: InProgAction) -> ModManagerResult<()> {
        match action.deref() {
            Action::Add(key) => {
                // Remove the mod that is partially enabled.
                self.db.remove_variant(key)?;
            },
            Action::Remove(key) => {
                // Continue with the deletion of the mod variant.
                self.db.remove_variant(key)?;
            },
        }

        // We finished cleaning up the in progress action, so now we can remove it from
        // disk.
        self.db.remove_in_prog_action()?;

        Ok(())
    }
}

impl From<SwappableAssetConflict> for SlotInfo {
    fn from(value: SwappableAssetConflict) -> Self {
        todo!()
    }
}
