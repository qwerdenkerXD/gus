// used types
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::Result;
use super::{
    TruePrimitiveType,
    StorageHandler,
    ModelName,
    TrueType,
    Record
};

#[allow(non_camel_case_types)]
#[derive(serde_derive::Serialize, Debug, clap::ValueEnum, Clone)]
pub enum StorageTypes {
    json
}

pub fn get_handler(storage_type: &StorageTypes, model_name: &ModelName) -> impl StorageHandler {
    match storage_type {
        StorageTypes::json => JsonStorageHandler {
            model_name: model_name.clone(),
        },
    }
}

struct JsonStorageHandler {
    model_name: ModelName
}

impl JsonStorageHandler {
    fn read_db(&self) -> HashMap<TrueType, Record> {
        unimplemented!()
    }
    fn save(&self) {
        todo!();
    }
}

impl StorageHandler for JsonStorageHandler {
    fn create_one(&self, id: &TrueType, record: &Record) -> Result<Record> {
        todo!();
    }
    fn read_one(&self, id: &TrueType) -> Result<Record> {
        todo!();
    }
    fn update_one(&self, id: &TrueType, record: Record) -> Result<Record> {
        todo!();
    }
    fn delete_one(&self, id: &TrueType) -> Result<Record> {
        todo!();
    }
}