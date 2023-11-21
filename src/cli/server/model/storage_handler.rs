mod json;

use crate::cli;

use std::path::PathBuf;
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
use dialoguer::{
    theme::ColorfulTheme,
    Confirm,
    Select
};

use std::fs::read_to_string;
use std::fs::write;
use serde_json::{
    to_string_pretty,
    from_str
};

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
    if let Some(start) = cli::get_valid_start_args() {
        let storage_config: StorageConfig = get_storage_configs(start.storage_definitions)?;
        return match model.storage_type {
            StorageType::json =>
                Ok(
                    JsonStorageHandler {
                        model_name: model.model_name.clone(),
                        key_attr: model.primary_key.clone(),
                        config: storage_config.json.unwrap()
                    }
                ),
        };
    }
    todo!("getting storage handlers is currently only possible when the server is running")
}

fn get_storage_configs(storage_file_path: Option<PathBuf>) -> Result<StorageConfig> {
    let mut storage_configs = StorageConfig {
        json: None
    };
    if let Some(path_buf) = storage_file_path {
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

    Ok(storage_configs)
}

pub fn configure_storages(args: cli::ConfigureStorages) {
    let mut configs: StorageConfig = match get_storage_configs(Some(args.storage_definitions.clone())) {
        Ok(configs) => configs,
        Err(_) => get_storage_configs(None).unwrap()
    };

    loop {
        let possible_storage_types = &[
            "json"
        ];
        let type_selection: usize = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Storage Type:")
            .default(0)
            .items(possible_storage_types)
            .interact()
            .unwrap();
        println!();
        match possible_storage_types[type_selection] {
            "json" => configs.json = Some(json::json_cli::configure_storage()),
            _ => unreachable!("All possible storage types have to be handled here")
        }
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to configure another storage type?")
            .interact()
            .unwrap()
        {
            break;
        }
    }

    // try to write the config to a file, else write it to stdout
    if write(args.storage_definitions, to_string_pretty(&configs).unwrap()).is_err() {
        println!("{config}", config=to_string_pretty(&configs).unwrap());
        eprintln!("unable to write file");
    }
}