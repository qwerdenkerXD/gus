// used types
use super::JsonStorageConfig;
use std::path::PathBuf;
use dialoguer::{
    theme::ColorfulTheme,
    Validator,
    Input
};

pub fn configure_storage() -> JsonStorageConfig {
    // get storage file path
    let storage_file_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Storage File Path:")
        .validate_with(PathValidator)
        .interact_text()
        .unwrap();

    JsonStorageConfig {
        storage_file: Some(PathBuf::from(storage_file_path))
    }
}

struct PathValidator;

impl Validator<String> for PathValidator {
    type Err = String;

    fn validate(&mut self, input: &String) -> Result<(), Self::Err> {
        let path = PathBuf::from(input);
        if path.is_dir() || path.file_name().is_none() {
            return Err("Expected file path".to_string());
        }
        match path.parent() {
            Some(parent) => {
                if !parent.is_dir() {
                    return Err("The file's parent directory does not exist".to_string());
                }
                Ok(())
            },
            // if input empty
            None => Err("Expected file path".to_string())
        }
    }
}