use ultimate_mod_man_rs_scraper::banana_scraper::BananaClient;
use ultimate_mod_man_rs_utils::types::ModIdentifier;

use crate::{
    mod_db::ModDb, mod_manager::ModManagerResult, mod_name_resolver::BananaModNameResolver,
};

pub(crate) async fn add_mod(
    ident: ModIdentifier,
    client: &BananaClient,
    resolver: &mut BananaModNameResolver,
    mod_db: &mut ModDb,
) -> ModManagerResult<()> {
    todo!()
}
