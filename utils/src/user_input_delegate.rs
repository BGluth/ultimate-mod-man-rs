use std::fmt::Display;

use crate::types::{
    PickedNonSwappableResolutionOption, PickedResolutionOption, PickedSwapOption, VariantAndId,
};

#[derive(Debug)]
pub struct VariantConflictSummary {
    new_variant: VariantAndId,
    existing_variant: VariantAndId,
}

pub struct SlotInfo {
    /// Human readable name that describes the slot (eg. "Banjo & Kazooie",
    /// "PS2").
    slot_name: String,

    /// Human readable name that describes the type (eg. "Character skin",
    /// "Stage skin").
    slot_type_name: String,
}

#[derive(Debug)]
pub struct AvailableSlotToSwapInto {
    slot_idx: usize,
    occupied_by: Option<VariantAndId>,
}

pub trait UserInputDelegate {
    fn get_yes_no_resp(&mut self) -> bool;

    /// The list provide is guaranteed to always have at least one element.
    fn select_item_from_list<T: Display>(&mut self, items: &[T]) -> usize;

    fn display_variant_conflict_summary(&mut self, summary: &VariantConflictSummary);

    fn choose_slot_to_swap_to(
        &mut self,
        slot_info: &SlotInfo,
        available_slots: &[AvailableSlotToSwapInto],
    ) -> PickedSwapOption;

    fn get_variant_conflict_resolution_option_swappable(
        &mut self,
        existing: &VariantAndId,
        new: &VariantAndId,
        slot_info: &SlotInfo,
        num_open_slots_available: usize,
    ) -> PickedResolutionOption;

    fn get_variant_conflict_resolution_option_non_swappable(
        &mut self,
        existing: &VariantAndId,
        new: &VariantAndId,
        slot_info: &SlotInfo,
    ) -> PickedNonSwappableResolutionOption;
}
