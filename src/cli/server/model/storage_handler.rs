mod json;

use json::JsonStorageHandler;
use std::io::Result;
use serde_derive::{
    Deserialize,
    Serialize
};
use super::{
    ModelName,
    TrueType,
    Record
};


#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
// if adding storage types, add them in the create-model dialogue in cli.rs too
pub enum StorageType {
    json
}

pub trait StorageHandler {
    fn create_one(&self, id: &TrueType, record: &Record) -> Result<Record>;
    fn read_one(&self, id: &TrueType) -> Result<Record>;
    fn update_one(&self, id: &TrueType, record: &Record) -> Result<Record>;
    fn delete_one(&self, id: &TrueType) -> Result<Record>;
}

pub fn get_handler(storage_type: &StorageType, model_name: &ModelName) -> impl StorageHandler {
    match storage_type {
        StorageType::json => JsonStorageHandler {
            model_name: model_name.clone(),
        },
    }
}