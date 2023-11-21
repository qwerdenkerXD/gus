pub mod json_cli;

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
    AttrName,
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
    pub key_attr: AttrName,
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
                            return Err(Error::new(ErrorKind::InvalidData, "Invalid storage file"));
                        }
                    }
                }
            },
            Err(err) => {
                match err.kind() {
                    ErrorKind::NotFound => (),
                    other => return Err(Error::new(other, format!("Unable to read storage file {path}", path=storage_file.display()).as_str()))
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
            return Err(Error::new(ErrorKind::PermissionDenied, format!("Unable to write data to storage file {path}", path=storage_file.display()).as_str()));
        }

        Ok(())
    }
}

impl StorageHandler for JsonStorageHandler {
    fn create_one(&self, record: &Record) -> Result<Record> {
        let id_string: String = to_string(record.get(&self.key_attr).unwrap()).unwrap();
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
            None => Err(Error::new(ErrorKind::NotFound, format!("No record found with id: {id_string}").as_str())),
        }
    }
    fn update_one(&self, record: &Record) -> Result<Record> {
        let id_string: String = to_string(record.get(&self.key_attr).unwrap()).unwrap();
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
            return Err(Error::new(ErrorKind::NotFound, format!("No record found to remove with id: {id_string}").as_str()));
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
            assert!(remove_file(file_name).is_ok(), "Storage file {file_name} already existing, unable to remove");
        }
    }

    fn post_test(file_name: &str) {
        if PathBuf::from(file_name).as_path().is_file() {
            assert!(remove_file(file_name).is_ok(), "Unable to remove storage file {file_name} after test");
        }
    }

    #[test]
    fn test_read_db() {
        const TEST_STORAGE_FILE: &str = "test_read_db.json";

        pre_test(TEST_STORAGE_FILE);
        let handler = JsonStorageHandler {
            model_name: ModelName(AttrName("movie".to_string())),
            key_attr: AttrName("id".to_string()),
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
        assert_eq!(&db.unwrap(), &expected, "Expected HashMap {map} when reading not existing storage file", map="{<model name>: {}}");

        // storage file empty
        // create file instead of write because it is not interpreted as existing
        assert!(write(TEST_STORAGE_FILE, "").is_ok(), "Unable to write storage file for tests");
        db = handler.read_db();
        assert!(db.is_ok(), "Unexpected Error after reading from empty storage file");
        assert_eq!(&db.unwrap(), &expected, "Expected HashMap {map} when reading from empty storage file", map="{<model name>: {}}");

        // storage file with data, movie missing
        assert!(write(TEST_STORAGE_FILE, "{\"another\": {\"1\": {\"id\": 1}}}").is_ok(), "Unable to write storage file for tests");
        db = handler.read_db();
        assert!(db.is_ok(), "Unexpected Error after reading from valid storage file with no respective model data");
        expected = HashMap::from([
            (ModelName(AttrName("another".to_string())), HashMap::from([
                    (
                        "1".to_string(),
                        Record::from([(AttrName("id".to_string()), TrueType::Primitive(Some(TruePrimitiveType::Integer(1))))])
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

    #[test]
    fn test_create_one() {
        const TEST_STORAGE_FILE: &str = "test_create_one.json";

        pre_test(TEST_STORAGE_FILE);
        let handler = JsonStorageHandler {
            model_name: ModelName(AttrName("movie".to_string())),
            key_attr: AttrName("id".to_string()),
            config: JsonStorageConfig {
                storage_file: Some(PathBuf::from(TEST_STORAGE_FILE))
            }
        };
        for key in ["1", "\"1\"", "true"] {
            let record = Record::from([
                (AttrName("id".to_string()), from_str::<TrueType>(key).unwrap()),
                (AttrName("name".to_string()), TrueType::Primitive(Some(TruePrimitiveType::String("Natural Born Killers".to_string())))),
                (AttrName("year".to_string()), TrueType::Primitive(Some(TruePrimitiveType::Integer(1994)))),
                (AttrName("actors".to_string()), TrueType::Array(Some(vec![TruePrimitiveType::String("Woody Harrelson".to_string()), TruePrimitiveType::String("Juliette Lewis".to_string())]))),
                (AttrName("recommended".to_string()), TrueType::Primitive(Some(TruePrimitiveType::Boolean(true))))
            ]);
            assert_eq!(handler.create_one(&record).unwrap(), record, "Creating a valid new record failed");
            assert!(handler.create_one(&record).is_err(), "Created a new record with already existing id");
        }

        post_test(TEST_STORAGE_FILE);
    }

    #[test]
    fn test_read_one() {
        const TEST_STORAGE_FILE: &str = "test_read_one.json";

        pre_test(TEST_STORAGE_FILE);
        let handler = JsonStorageHandler {
            model_name: ModelName(AttrName("movie".to_string())),
            key_attr: AttrName("id".to_string()),
            config: JsonStorageConfig {
                storage_file: Some(PathBuf::from(TEST_STORAGE_FILE))
            }
        };
        for key in ["1", "\"1\"", "true"] {
            assert!(write(TEST_STORAGE_FILE, format!("{{\"movie\": {{ {key:?} : {{ \"id\":{key} }} }} }}")).is_ok(), "Unable to write storage file for tests");
            let id: TrueType = from_str(key).unwrap();
            let record = Record::from([
                (AttrName("id".to_string()), id.clone())
            ]);
            assert_eq!(handler.read_one(&id).unwrap(), record, "Reading a valid new record failed");
        }

        assert!(handler.read_one(&from_str::<TrueType>("\"not existing\"").unwrap()).is_err(), "Expected error when reading from a not existing file");

        post_test(TEST_STORAGE_FILE);
    }

    #[test]
    fn test_update_one() {
        const TEST_STORAGE_FILE: &str = "test_update_one.json";

        pre_test(TEST_STORAGE_FILE);
        let handler = JsonStorageHandler {
            model_name: ModelName(AttrName("movie".to_string())),
            key_attr: AttrName("id".to_string()),
            config: JsonStorageConfig {
                storage_file: Some(PathBuf::from(TEST_STORAGE_FILE))
            }
        };
        for key in ["1", "\"1\"", "true"] {
            assert!(write(TEST_STORAGE_FILE, format!("{{\"movie\": {{ {key:?} : {{ \"id\":\"dummy\" }} }} }}")).is_ok(), "Unable to write storage file for tests");
            let id: TrueType = from_str(key).unwrap();
            let record = Record::from([
                (AttrName("id".to_string()), id.clone())
            ]);
            assert_eq!(handler.update_one(&record).unwrap(), record, "Updating an existing record failed");

            let record = Record::from([
                (AttrName("id".to_string()), from_str::<TrueType>("\"not existing\"").unwrap())
            ]);
            assert!(handler.update_one(&record).is_err(), "Expected an error when updating a not existing record");
        }

        post_test(TEST_STORAGE_FILE);
    }

    #[test]
    fn test_delete_one() {
        const TEST_STORAGE_FILE: &str = "test_delete_one.json";

        pre_test(TEST_STORAGE_FILE);
        let handler = JsonStorageHandler {
            model_name: ModelName(AttrName("movie".to_string())),
            key_attr: AttrName("id".to_string()),
            config: JsonStorageConfig {
                storage_file: Some(PathBuf::from(TEST_STORAGE_FILE))
            }
        };
        for key in ["1", "\"1\"", "true"] {
            assert!(write(TEST_STORAGE_FILE, format!("{{\"movie\": {{ {key:?} : {{ \"id\":{key} }} }} }}")).is_ok(), "Unable to write storage file for tests");
            let id: TrueType = from_str(key).unwrap();
            let record = Record::from([
                (AttrName("id".to_string()), id.clone())
            ]);
            assert_eq!(handler.delete_one(&id).unwrap(), record, "Deleting a valid new record failed");
        }

        assert!(handler.delete_one(&from_str::<TrueType>("\"not existing\"").unwrap()).is_err(), "Expected error when deleting from a not existing file");

        post_test(TEST_STORAGE_FILE);
    }
}