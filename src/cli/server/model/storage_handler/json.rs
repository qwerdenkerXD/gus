// used types
use std::collections::HashMap;
use std::path::PathBuf;
use serde_derive::{
    Deserialize,
    Serialize
};
use std::io::{
    ErrorKind,
    Result,
    Error
};
use super::StorageHandler;
use super::super::{
    ModelName,
    TrueType,
    Record
};
use std::fs::{
    read_to_string,
    write
};
use serde_json::{
    to_string,
    from_str
};

const DEFAULT_STORAGE_FILE: &str = "./data.json.gus";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JsonStorageConfig {
    pub storage_file: Option<PathBuf>
}

pub struct JsonStorageHandler {
    pub model_name: ModelName,
    pub config: JsonStorageConfig
}

impl JsonStorageHandler {
    fn read_db(&self) -> Result<HashMap<ModelName, HashMap<String, Record>>> {
        let storage_file: &PathBuf = &self.config.storage_file.clone().unwrap_or(PathBuf::from(DEFAULT_STORAGE_FILE));
        let mut db: HashMap<ModelName, HashMap<String, Record>> = HashMap::new();
        match read_to_string(storage_file) {
            Ok(data) => {
                match from_str(&data) {
                    Ok(parsed) => db = parsed,
                    Err(err) => {
                        if !err.is_eof() {
                            return Err(err.into());
                        }
                    }
                }
            },
            Err(err) => {
                match err.kind() {
                    ErrorKind::NotFound => (),
                    other => return Err(Error::new(other, format!("Unable to read storage file {}", storage_file.display()).as_str()))
                }
            }
        }

        if db.get(&self.model_name).is_none() {
            db.insert(self.model_name.clone(), HashMap::new());
        }

        Ok(db.clone())
    }
    fn save(&self, db: &mut HashMap<ModelName, HashMap<String, Record>>) -> Result<()> {
        let storage_file: &PathBuf = &self.config.storage_file.clone().unwrap_or(PathBuf::from(DEFAULT_STORAGE_FILE));
        if write(storage_file, to_string(db).unwrap()).is_err() {
            return Err(Error::new(ErrorKind::PermissionDenied, format!("Unable to write data to storage file {}", storage_file.display()).as_str()));
        }

        Ok(())
    }
}

impl StorageHandler for JsonStorageHandler {
    fn create_one(&self, id: &TrueType, record: &Record) -> Result<Record> {
        let id_string: String = to_string(id).unwrap();
        let db = &mut self.read_db()?;
        let mut data: HashMap<String, Record> = db.get(&self.model_name).unwrap().clone();
        if data.get(&id_string).is_some() {
            return Err(Error::new(ErrorKind::AlreadyExists, "A record for the given key already exists, try to update it instead (PUT)"));
        }
        data.insert(id_string, record.clone());
        db.insert(self.model_name.clone(), data);
        self.save(db)?;

        Ok(record.clone())
    }
    fn read_one(&self, id: &TrueType) -> Result<Record> {
        let id_string: &String = &to_string(id).unwrap();
        let db = self.read_db()?;
        let data: HashMap<String, Record> = db.get(&self.model_name).unwrap().clone();
        match data.get(id_string) {
            Some(record) => Ok(record.clone()),
            None => Err(Error::new(ErrorKind::NotFound, format!("No record found with id: {}", id_string).as_str())),
        }
    }
    fn update_one(&self, id: &TrueType, record: &Record) -> Result<Record> {
        let id_string: String = to_string(id).unwrap();
        let db = &mut self.read_db()?;
        let mut data: HashMap<String, Record> = db.get(&self.model_name).unwrap().clone();
        let mut new_record: Record;
        if let Some(orig_record) = data.get(&id_string) {
            new_record = orig_record.clone();
            for (key, value) in record {
                new_record.insert(key.clone(), value.clone());
            }
        } else {
            return Err(Error::new(ErrorKind::NotFound, "No record found for the given key, try to create it instead (POST)"));
        }

        data.insert(id_string, new_record.clone());
        db.insert(self.model_name.clone(), data);
        self.save(db)?;

        Ok(new_record)
    }
    fn delete_one(&self, id: &TrueType) -> Result<Record> {
        let id_string: String = to_string(id).unwrap();
        let db = &mut self.read_db()?;
        let mut data: HashMap<String, Record> = db.get(&self.model_name).unwrap().clone();
        let record: Option<Record> = data.remove(&id_string);
        if record.is_none() {
            return Err(Error::new(ErrorKind::NotFound, format!("No record found to remove with id: {}", id_string).as_str()));
        }
        db.insert(self.model_name.clone(), data);
        self.save(db)?;

        Ok(record.unwrap())
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::server::model::{
        TruePrimitiveType,
        AttrName
    };

    use std::fs::remove_file;

    fn pre_test(file_name: &str) {
        if PathBuf::from(file_name).as_path().is_file() {
            assert!(remove_file(file_name).is_ok(), "Storage file {} already existing, unable to remove", file_name);
        }
    }

    fn post_test(file_name: &str) {
        if PathBuf::from(file_name).as_path().is_file() {
            assert!(remove_file(file_name).is_ok(), "Unable to remove storage file {} after test", file_name);
        }
    }

    #[test]
    fn test_read_db() {
        const TEST_STORAGE_FILE: &'static str = "test_read_db.json";

        pre_test(TEST_STORAGE_FILE);
        let handler = JsonStorageHandler {
            model_name: ModelName(AttrName("movie".to_string())),
            config: JsonStorageConfig {
                storage_file: Some(PathBuf::from(TEST_STORAGE_FILE))
            }
        };

        // storage file doesn't exist
        let mut db: Result<HashMap<ModelName, HashMap<String, Record>>> = handler.read_db();
        assert!(db.is_ok(), "Unexpected Error when reading not existing storage file");
        let mut expected: HashMap<ModelName, HashMap<String, Record>> = HashMap::from([
            (handler.model_name.clone(), HashMap::new())
        ]);
        assert_eq!(&db.unwrap(), &expected, "Expected HashMap {} when reading not existing storage file", "{<model name>: {}}");

        // storage file empty
        // create file instead of write because it is not interpreted as existing
        assert!(write(TEST_STORAGE_FILE, "").is_ok(), "Unable to write storage file for tests");
        db = handler.read_db();
        assert!(db.is_ok(), "Unexpected Error after reading from empty storage file");
        assert_eq!(&db.unwrap(), &expected, "Expected HashMap {} when reading from empty storage file", "{<model name>: {}}");

        // storage file with data, movie missing
        assert!(write(TEST_STORAGE_FILE, "{\"another\": {\"1\": {\"id\": 1}}}").is_ok(), "Unable to write storage file for tests");
        db = handler.read_db();
        assert!(db.is_ok(), "Unexpected Error after reading from valid storage file with no respective model data");
        expected = HashMap::from([
            (ModelName(AttrName("another".to_string())), HashMap::from([
                    (
                        "1".to_string(),
                        HashMap::from([(AttrName("id".to_string()), TrueType::Primitive(TruePrimitiveType::Integer(1)))])
                    )
                ])
            ),
            (handler.model_name.clone(), HashMap::new())
        ]);
        assert_eq!(&db.unwrap(), &expected, "Gotten HashMap doesn't match the expected when reading from valid storage file with data but no respective model data");

        // storage file not JSON
        assert!(write(TEST_STORAGE_FILE, "i am not json {\"id\":false}").is_ok(), "Unable to write storage file for tests");
        db = handler.read_db();
        assert!(db.is_err(), "Expected Error after reading from invalid storage file");

        post_test(TEST_STORAGE_FILE);
    }

    // test not completed, testing just the basic creation
    // reading empty data file is also sth. to test
    // also test all primary key types
    #[test]
    fn test_create_one() {
        const TEST_STORAGE_FILE: &'static str = "test_create_one.json";

        pre_test(TEST_STORAGE_FILE);
        let handler = JsonStorageHandler {
            model_name: ModelName(AttrName("movie".to_string())),
            config: JsonStorageConfig {
                storage_file: Some(PathBuf::from(TEST_STORAGE_FILE))
            }
        };
        let record: Record = HashMap::from([
            (AttrName("id".to_string()), TrueType::Primitive(TruePrimitiveType::Integer(1))),
            (AttrName("name".to_string()), TrueType::Primitive(TruePrimitiveType::String("Natural Born Killers".to_string()))),
            (AttrName("year".to_string()), TrueType::Primitive(TruePrimitiveType::Integer(1994))),
            (AttrName("actors".to_string()), TrueType::Array(vec![TruePrimitiveType::String("Woody Harrelson".to_string()), TruePrimitiveType::String("Juliette Lewis".to_string())])),
            (AttrName("recommended".to_string()), TrueType::Primitive(TruePrimitiveType::Boolean(true)))
        ]);
        assert_eq!(handler.create_one(&TrueType::Primitive(TruePrimitiveType::Integer(1)), &record).unwrap(), record, "Creating a valid new record failed");
        assert!(handler.create_one(&TrueType::Primitive(TruePrimitiveType::Integer(1)), &record).is_err(), "Created a new record with already existing id");

        post_test(TEST_STORAGE_FILE);
    }
}