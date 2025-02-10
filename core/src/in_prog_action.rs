use std::{
    fmt::{self, Display, Formatter},
    fs, io,
    ops::Deref,
};

use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ultimate_mod_man_rs_utils::{
    types::VariantAndId,
    utils::{
        DeserializationError, SerializationError, deserialize_data_from_path,
        serialize_data_to_path,
    },
};

type InProgActionResult<T> = Result<T, InProgActionError>;

#[derive(Debug, Error)]
pub enum InProgActionError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    DeserializationError(#[from] DeserializationError),

    #[error(transparent)]
    SerializationError(#[from] SerializationError),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum Action {
    Add(VariantAndId),
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Action::Add(key) => write!(f, "Add - ({})", key),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InProgAction {
    in_prog: Action,
}

impl Deref for InProgAction {
    type Target = Action;

    fn deref(&self) -> &Self::Target {
        &self.in_prog
    }
}

impl InProgAction {
    pub(crate) fn new(action: Action) -> Self {
        Self { in_prog: action }
    }

    pub(crate) fn load_from_disk_if_present(p: &Utf8Path) -> InProgActionResult<Option<Self>> {
        Ok(match fs::exists(p)? {
            false => None,
            true => Some(deserialize_data_from_path(p)?),
        })
    }

    pub(crate) fn sync_to_disk(&self, p: &Utf8Path) -> InProgActionResult<()> {
        serialize_data_to_path(p, self)?;
        Ok(())
    }
}
