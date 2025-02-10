use std::{fs, io};

use camino::Utf8Path;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

pub type DeserializationResult<T> = Result<T, DeserializationError>;
pub type SerializationResult<T> = Result<T, SerializationError>;

#[derive(Debug, Error)]
pub enum DeserializationError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Deserialization(#[from] toml::de::Error),
}

#[derive(Debug, Error)]
pub enum SerializationError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Serialization(#[from] toml::ser::Error),
}

pub fn deserialize_data_from_path<T: DeserializeOwned>(p: &Utf8Path) -> DeserializationResult<T> {
    toml::from_str(&fs::read_to_string(p)?).map_err(|e| e.into())
}

pub fn serialize_data_to_path<T: Serialize>(p: &Utf8Path, v: &T) -> SerializationResult<()> {
    fs::write(p, &toml::to_string(v)?)?;
    Ok(())
}
