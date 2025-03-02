//! I want some automated way to detect if the scraping logic ever breaks. I
//! don't think these need to be run with the standard batter of tests, but
//! instead I might setup some automated job that runs every day just to detect
//! when this breaks.
use ultimate_mod_man_rs_scraper::banana_scraper::BananaClient;
use ultimate_mod_man_rs_utils::{
    types::{ModId, VariantAndId},
    user_input_delegate::UserInputDelegate,
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
