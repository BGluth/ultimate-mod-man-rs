use std::fmt::Display;

use crate::types::{AssetSlot, VariantAndId};

pub type SwapPickedIndex = usize;

#[derive(Debug)]
pub enum PickedResolutionOptionSwappable {
    NonSwapOption(PickedResolutionOptionNonSwappable),
    Swap(SwapPickedIndex),
}

#[derive(Debug)]
pub enum PickedResolutionOptionNonSwappable {
    KeepExisting,
    Replace,
}

#[derive(Debug)]
pub struct VariantConflictSummary {
    new_variant: VariantAndId,
    conflicting_variants: Vec<ConflictingVariant>,
}

#[derive(Debug)]
pub struct ConflictingVariant {
    key: VariantAndId,
    conflicting_slots: Vec<ConflictingSlot>,
}

#[derive(Debug)]
pub struct ConflictingSlot {
    slot: AssetSlot,
    num_files: usize,
}

pub trait UserInputDelegate {
    fn get_yes_no_resp(&mut self) -> bool;

    /// The list provide is guaranteed to always have at least one element.
    fn select_item_from_list<T: Display>(&mut self, items: &[T]) -> usize;

    fn display_variant_conflict_summary(&mut self, summary: &VariantConflictSummary);

    fn get_variant_conflict_resolution_option_swappable<T: Display>(
        &mut self,
        existing: &VariantAndId,
        new: &VariantAndId,
        slot: AssetSlot,
        available_slots: &[T],
    ) -> PickedResolutionOptionSwappable;

    fn get_variant_conflict_resolution_option_non_swappable<T: Display>(
        &mut self,
        existing: &VariantAndId,
        new: &VariantAndId,
        slot: AssetSlot,
    ) -> PickedResolutionOptionNonSwappable;
}
