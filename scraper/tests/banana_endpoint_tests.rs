//! I want some automated way to detect if the scraping logic ever breaks. I don't think these need to be run with the standard batter of tests, but instead I might setup some automated job that runs every day just to detect when this breaks.
use ultimate_mod_man_rs_scraper::{
    banana_scraper::{download_mod_variant, resolve_mod_name},
    types::ModId,
};

static BULLEY_MAGUIRE_MOD_NAME: &str = "BULLY MAGUIRE over Joker";
static BULLEY_MAGUIRE_MOD_VARIANT: &str = "tobeymaguire_85f71.zip";
const BULLEY_MAGUIRE_MOD_ID: ModId = 378330;

// TODO: Add a special compile time flag to prevent these tests from running with other tests...
#[test]
fn resolve_mod_name_works_works() {
    let res = resolve_mod_name(BULLEY_MAGUIRE_MOD_NAME);

    assert!(res.is_ok());
    assert!(matches!(res, Ok(BULLEY_MAGUIRE_MOD_ID)));
}

#[test]
fn download_variant_works() {
    let res = download_mod_variant(BULLEY_MAGUIRE_MOD_ID, BULLEY_MAGUIRE_MOD_VARIANT);
    assert!(res.is_ok());
}
