// used types
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::{
    ErrorKind,
    Result,
    Error
};
use super::{
    ModelName,
    TrueType,
    Record
};
use std::fs::{
    write,
    read_to_string
};
use serde_json::{
    to_string,
    from_str
};

#[allow(non_camel_case_types)]
#[derive(serde_derive::Serialize, Debug, clap::ValueEnum, Clone)]
pub enum StorageTypes {
    json
}

pub trait StorageHandler {
    fn create_one(&self, id: &TrueType, record: &Record) -> Result<Record>;
    fn read_one(&self, id: &TrueType) -> Result<Record>;
    fn update_one(&self, id: &TrueType, record: Record) -> Result<Record>;
    fn delete_one(&self, id: &TrueType) -> Result<Record>;
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
    fn read_db(&self) -> Result<HashMap<ModelName, HashMap<String, Record>>> {
        let storage_file: &str = "data.json.gus";
        if let Ok(data) = read_to_string(storage_file) {
            let mut db = from_str::<HashMap<ModelName, HashMap<String, Record>>>(&data)?;
            if db.get(&self.model_name).is_none() {
                db.insert(self.model_name.clone(), HashMap::new());
            }
            return Ok(db.clone())
        }
        if PathBuf::from(storage_file).as_path().is_file() {
            return Err(Error::new(ErrorKind::PermissionDenied, "Unable to read storage file ./data.json.gus"));
        }
        match write(storage_file, "") {
            Ok(_) => return Ok(HashMap::from([
                (self.model_name.clone(), HashMap::new())
            ])),
            Err(err) => return Err(Error::new(ErrorKind::PermissionDenied, "Unable to create storage file ./data.json.gus"))
        }
    }
    fn save(&self, db: &mut HashMap<ModelName, HashMap<String, Record>>) -> Result<()> {
        if let Err(err) = write("./data.json.gus", &to_string(db).unwrap()) {
            return Err(Error::new(ErrorKind::PermissionDenied, "Unable to write data to storage file ./data.json.gus"));
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
            return Err(Error::new(ErrorKind::AlreadyExists, "A record for the given key already exists, try to update it instead"));
        }
        data.insert(id_string, record.clone());
        db.insert(self.model_name.clone(), data);
        self.save(db)?;
        return Ok(record.clone());
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




#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{
        TruePrimitiveType,
        AttrName
    };

    #[test]
    // test not completed, testing just the basic creation
    // reading empty data file is also sth. to test
    // also test all primary key types
    fn test_json_create_one() {
        let handler = JsonStorageHandler {
            model_name: ModelName(AttrName("movie".to_string()))
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
    }
}