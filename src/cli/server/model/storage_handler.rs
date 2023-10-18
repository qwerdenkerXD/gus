mod json;

use crate::cli;

use std::io::{
    ErrorKind,
    Result,
    Error
};
use json::{
    JsonStorageHandler,
    JsonStorageConfig
};
use serde_derive::{
    Deserialize,
    Serialize
};
use super::{
    ModelDefinition,
    TrueType,
    Record
};

use std::fs::read_to_string;
use serde_json::from_str;

#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Debug, PartialEq, Hash, Eq, Clone)]
// if adding storage types, add them in the create-model dialogue in cli.rs too
pub enum StorageType {
    json
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct StorageConfig {
    json: Option<JsonStorageConfig>
}

pub trait StorageHandler {
    fn create_one(&self, record: &Record) -> Result<Record>;
    fn read_one(&self, id: &TrueType) -> Result<Record>;
    fn update_one(&self, record: &Record) -> Result<Record>;
    fn delete_one(&self, id: &TrueType) -> Result<Record>;
}

pub fn get_handler(model: &ModelDefinition) -> Result<impl StorageHandler> {
    let storage_config: StorageConfig = get_storage_configs()?;
    match model.storage_type {
        StorageType::json =>
            Ok(
                JsonStorageHandler {
                    model_name: model.model_name.clone(),
                    key_attr: model.primary_key.clone(),
                    config: storage_config.json.unwrap()
                }
            ),
    }
}

fn get_storage_configs() -> Result<StorageConfig> {
    let mut storage_configs = StorageConfig {
        json: None
    };
    if let Some(start) = cli::get_valid_start_args() {
        if let Some(path_buf) = start.storage_definitions {
            let data: Result<String> = read_to_string(path_buf.as_path());
            if data.is_err() {
                return Err(Error::new(ErrorKind::PermissionDenied, "Unable to read storage definition file, make sure it's utf-8 only"))
            }
            match from_str::<StorageConfig>(&data.unwrap()) {
                Ok(storages) => storage_configs = storages,
                Err(err) => if !err.is_eof() {
                    return Err(Error::new(ErrorKind::InvalidData, "The storage definition file is invalid"));
                }
            }
        }

        // set defaults if None
        if storage_configs.json.is_none() {
            storage_configs.json = Some(
                JsonStorageConfig {
                    storage_file: None
                }
            );
        }

        return Ok(storage_configs);
    }
    todo!("getting storage configs is currently only possible when the server is running")
}