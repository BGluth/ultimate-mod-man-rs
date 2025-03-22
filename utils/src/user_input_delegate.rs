use std::{fmt::Display, path::PathBuf};

use crate::types::{
    AssetSlot, AvailableSlotsToSwapToInfo, PickedNonSwappableResolutionOption,
    PickedResolutionOption, SkinSlotValue, StageSlotValue, VariantAndId,
};

#[derive(Debug)]
pub struct VariantConflictSummary {
    new_variant: VariantAndId,
    conflicting_slot_info: ConflictingSlotInfo,
    num_files: usize,
}

#[derive(Debug)]
pub enum ConflictingSlotInfo {
    Skin(ConflictingSkinSlotInfo),
    Stage(StageSlotValue),
    Global(PathBuf),
}

#[derive(Debug)]
pub struct ConflictingSkinSlotInfo {
    slot: SkinSlotValue,
    available_slots_to_swap_to: Vec<SkinSlotValue>,
}

pub trait UserInputDelegate {
    fn get_yes_no_resp(&mut self) -> bool;

    /// The list provide is guaranteed to always have at least one element.
    fn select_item_from_list<T: Display>(&mut self, items: &[T]) -> usize;

    fn display_variant_conflict_summary(&mut self, summary: &VariantConflictSummary);

    fn get_variant_conflict_resolution_option_swappable(
        &mut self,
        existing: &VariantAndId,
        new: &VariantAndId,
        slot: &AssetSlot,
        available_slots: &AvailableSlotsToSwapToInfo,
    ) -> PickedResolutionOption;

    fn get_variant_conflict_resolution_option_non_swappable(
        &mut self,
        existing: &VariantAndId,
        new: &VariantAndId,
        slot: &AssetSlot,
    ) -> PickedNonSwappableResolutionOption;
}
