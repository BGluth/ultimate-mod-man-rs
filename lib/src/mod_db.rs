use serde::{Deserialize, Serialize};

pub type ModId = u64;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ModDb {
    entries: Vec<ModEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModEntry {
    id: ModId,
    name: String,
}
