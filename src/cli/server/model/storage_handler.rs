mod json;

use json::JsonStorageHandler;
use std::io::Result;
use super::{
    ModelName,
    AttrName,
    TrueType,
    Record
};

#[allow(non_camel_case_types)]
#[derive(serde_derive::Serialize, Debug, clap::ValueEnum, Clone)]
pub enum StorageTypes {
    json
}

pub trait StorageHandler {
    fn create_one(&self, id: &TrueType, record: &Record) -> Result<Record>;
    fn read_one(&self, id: &TrueType) -> Result<Record>;
    fn update_one(&self, id: &TrueType, id_attr: &AttrName, record: &Record) -> Result<Record>;
    fn delete_one(&self, id: &TrueType) -> Result<Record>;
}

pub fn get_handler(storage_type: &StorageTypes, model_name: &ModelName) -> impl StorageHandler {
    match storage_type {
        StorageTypes::json => JsonStorageHandler {
            model_name: model_name.clone(),
        },
    }
}