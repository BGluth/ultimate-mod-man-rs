//! I want some automated way to detect if the scraping logic ever breaks. I
//! don't think these need to be run with the standard batter of tests, but
//! instead I might setup some automated job that runs every day just to detect
//! when this breaks.

use ultimate_mod_man_rs_scraper::banana_scraper::BananaClient;
use ultimate_mod_man_rs_utils::{
    types::{
        AssetSlot, AvailableSlotsToSwapToInfo, ModId, PickedNonSwappableResolutionOption,
        PickedResolutionOption, SwappableAssetSlot, VariantAndId,
    },
    user_input_delegate::{UserInputDelegate, VariantConflictSummary},
};

static BULLEY_MAGUIRE_MOD_NAME: &str = "BULLY MAGUIRE over Joker";
static BULLEY_MAGUIRE_MOD_VARIANT: &str = "tobeymaguire_85f71.zip";
const BULLEY_MAGUIRE_MOD_ID: ModId = 378330;

/// For testing.
///
/// Always returns "yes" and picks the first option in a list.
struct DummyDelegate {}

impl UserInputDelegate for DummyDelegate {
    fn get_yes_no_resp(&mut self) -> bool {
        true
    }

    fn select_item_from_list<T>(&mut self, items: &[T]) -> usize {
        0
    }

    fn display_variant_conflict_summary(&mut self, summary: &VariantConflictSummary) {}

    fn choose_slot_to_swap_to(
        &mut self,
        slot: ultimate_mod_man_rs_utils::types::SwappableAssetSlot,
        available_slots: &AvailableSlotsToSwapToInfo,
    ) -> ultimate_mod_man_rs_utils::types::PickedSwapOption {
        todo!()
    }

    fn get_variant_conflict_resolution_option_swappable(
        &mut self,
        existing: &VariantAndId,
        new: &VariantAndId,
        slot: &SwappableAssetSlot,
        available_slots: &AvailableSlotsToSwapToInfo,
    ) -> PickedResolutionOption {
        PickedResolutionOption::NonSwapOption(PickedNonSwappableResolutionOption::KeepExisting)
    }

    fn get_variant_conflict_resolution_option_non_swappable(
        &mut self,
        existing: &VariantAndId,
        new: &VariantAndId,
        slot: &AssetSlot,
    ) -> PickedNonSwappableResolutionOption {
        PickedNonSwappableResolutionOption::KeepExisting
    }
}

// TODO: Add a special compile time flag to prevent these tests from running
// with other tests...
#[tokio::test]
async fn resolve_mod_name_works_works() {
    let res = BananaClient::new()
        .unwrap()
        .resolve_mod_name(BULLEY_MAGUIRE_MOD_NAME)
        .await;

    assert!(res.is_ok());
    assert!(matches!(res, Ok(BULLEY_MAGUIRE_MOD_ID)));
}

#[tokio::test]
async fn download_variant_works() {
    let res = BananaClient::new()
        .unwrap()
        .download_mod_variant(
            &mut DummyDelegate {},
            &VariantAndId::new(
                BULLEY_MAGUIRE_MOD_ID,
                BULLEY_MAGUIRE_MOD_VARIANT.to_string(),
            ),
        )
        .await;

    assert!(res.is_ok());
}
