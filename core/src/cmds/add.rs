use ultimate_mod_man_rs_scraper::banana_scraper::BananaClient;

use crate::{
    mod_db::{ModDb, ModIdentifier},
    mod_manager::ModManagerResult,
    mod_name_resolver::BananaModNameResolver,
};

pub(crate) async fn add_mod(
    ident: ModIdentifier,
    client: &BananaClient,
    resolver: &mut BananaModNameResolver,
    mod_db: &mut ModDb,
) -> ModManagerResult<()> {
    let m_id = resolver.resolve_mod_ident(client, ident).await?;
    mod_db.add_mod(m_id);

    todo!()
}
