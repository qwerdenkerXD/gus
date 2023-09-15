use crate::cli::server::model::types::{
    TruePrimitiveType,
    StorageHandler,
    ModelName,
    Record
};
use std::path::PathBuf;
use std::io::Result;

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum StorageTypes {
    json
}

pub fn get_handler(storage_type: &StorageTypes, model_name: &ModelName, storage_file: &PathBuf) -> impl StorageHandler {
    match storage_type {
        StorageTypes::json => JsonStorageHandler {
            model_name: model_name.clone(),
            storage_file: storage_file.clone()
        },
    }
}

struct JsonStorageHandler {
    model_name: ModelName,
    storage_file: PathBuf
}

impl JsonStorageHandler {
    fn save(&self) {
        todo!();
    }
}

impl StorageHandler for JsonStorageHandler {
    fn create_one(&self, record: Record) -> Result<Record> {
        todo!();
    }
    fn read_one(&self, id: TruePrimitiveType) -> Result<Record> {
        todo!();
    }
    fn update_one(&self, id: TruePrimitiveType, record: Record) -> Result<Record> {
        todo!();
    }
    fn delete_one(&self, id: TruePrimitiveType) -> Result<Record> {
        todo!();
    }
}