use crate::{
    mod_db::{ModDb, ModIdentifier},
    mod_manager::ModManagerResult,
    mod_name_resolver::BananaModNameResolver,
};

pub(crate) fn add_mod(
    ident: ModIdentifier,
    resolver: &mut BananaModNameResolver,
    mod_db: &mut ModDb,
) -> ModManagerResult<()> {
    let m_id = resolver.resolve_mod_ident(ident);
    mod_db.add_mod(m_id);

    todo!()
}
