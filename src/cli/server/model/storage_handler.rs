mod json;

use crate::cli;

use std::collections::HashMap;
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
    ModelName,
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

type Storages = HashMap<StorageType, Option<StorageConfig>>;

#[derive(Deserialize, Serialize, Debug, Clone)]
enum StorageConfig {
    Json(JsonStorageConfig)
}

pub trait StorageHandler {
    fn create_one(&self, id: &TrueType, record: &Record) -> Result<Record>;
    fn read_one(&self, id: &TrueType) -> Result<Record>;
    fn update_one(&self, id: &TrueType, record: &Record) -> Result<Record>;
    fn delete_one(&self, id: &TrueType) -> Result<Record>;
}

pub fn get_handler(storage_type: &StorageType, model_name: &ModelName) -> Result<impl StorageHandler> {
    let storage_config: StorageConfig = get_storage_config(storage_type)?;
    match storage_config {
        StorageConfig::Json(config) =>
            Ok(
                JsonStorageHandler {
                    model_name: model_name.clone(),
                    config: config
                }
            ),
    }
}

fn get_storage_config(storage_type: &StorageType) -> Result<StorageConfig> {
    if let Some(start) = cli::get_valid_start_args() {
        if let Some(path_buf) = start.storage_definitions {
            let data: Result<String> = read_to_string(&path_buf.as_path());
            if data.is_err() {
                return Err(Error::new(ErrorKind::PermissionDenied, "Unable to read storage definition file, make sure it's utf-8 only"))
            }
            if let Ok(storages) = from_str::<Storages>(&data.unwrap()) {
                if let Some(storage) = storages.get(storage_type) {
                    if let Some(config) = storage {
                        return Ok(config.clone());
                    }
                }
            } else {
                return Err(Error::new(ErrorKind::InvalidData, "The storage definition file is invalid"))
            }
        }

        match storage_type {
            StorageType::json => return Ok(StorageConfig::Json(JsonStorageConfig {
                storage_file: None
            }))
        }
    }
    todo!("getting storage configs is currently only possible when the server is running")
}