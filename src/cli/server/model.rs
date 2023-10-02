mod types;
mod storage_handler;

pub mod model_cli;

//used modules
use crate::cli;

pub use types::*;
use storage_handler::*;

// used types
pub use storage_handler::StorageType;
use std::collections::HashMap;
use serde_json::Value;
use std::fs::ReadDir;
use std::path::Path;
use std::io::{
    ErrorKind,
    Result,
    Error
};
use ErrorKind::{
    InvalidData,
    NotFound
};

// used functions
use std::fs::{
    read_to_string,
    read_dir
};

pub fn create_one(model_name: &str, json: &str) -> Result<Record> {
    if let Some(args) = cli::get_valid_start_args() {
        let name: &ModelName = &ModelName(AttrName::try_from(model_name)?);
        let model: ModelDefinition = parse_model(args.modelspath.as_path(), name)?;
        let storage_handler = get_handler(&model.storage_type, name)?;
        let record: Record = parse_record(json, &model)?;
        return storage_handler.create_one(record.get(&model.primary_key).unwrap(), &record);
    };
    todo!("creating records is currently only possible when the server is running")
}

pub fn read_one(model_name: &str, id: &str) -> Result<Record> {
    if let Some(args) = cli::get_valid_start_args() {
        let name: &ModelName = &ModelName(AttrName::try_from(model_name)?);
        let model: ModelDefinition = parse_model(args.modelspath.as_path(), name)?;
        let storage_handler = get_handler(&model.storage_type, name)?;
        let true_id: &TrueType = &parse_id_str(id, &model)?;
        return storage_handler.read_one(true_id);
    };
    todo!("reading records is currently only possible when the server is running")
}

pub fn update_one(model_name: &str, id: &str, json: &str) -> Result<Record> {
    if let Some(args) = cli::get_valid_start_args() {
        let name: &ModelName = &ModelName(AttrName::try_from(model_name)?);
        let mut model: ModelDefinition = parse_model(args.modelspath.as_path(), name)?;
        let storage_handler = get_handler(&model.storage_type, name)?;
        let mut required: Vec<AttrName> = model.required;

        // parse record to get its attributes
        model.required = vec!();
        let record: Record = parse_record(json, &model)?;

        // update models' required attributes to the necessary
        required.retain(|a| record.get(a).is_some());
        model.required = required.clone();

        // parse the record again, this time with correct requirement check
        let mut valid_record: Record = parse_record(json, &model)?;
        let true_id: &TrueType = &parse_id_str(id, &model)?;
        valid_record.insert(model.primary_key.clone(), true_id.clone());
        return storage_handler.update_one(true_id, &record);
    };
    todo!("updating records is currently only possible when the server is running")
}

pub fn delete_one(model_name: &str, id: &str) -> Result<Record> {
    if let Some(args) = cli::get_valid_start_args() {
        let name: &ModelName = &ModelName(AttrName::try_from(model_name)?);
        let model: ModelDefinition = parse_model(args.modelspath.as_path(), name)?;
        let storage_handler = get_handler(&model.storage_type, name)?;
        let true_id: &TrueType = &parse_id_str(id, &model)?;
        return storage_handler.delete_one(true_id);
    };
    todo!("reading records is currently only possible when the server is running")
}

fn parse_id_str(id: &str, model: &ModelDefinition) -> Result<TrueType> {
    let key: &AttrName = &model.primary_key;
    let key_type: &AttrType = model.attributes.get(key).unwrap();

    match key_type {
        AttrType::Primitive(PrimitiveType::String) => { 
            let value: Value = parse(format!("{:?}", id).as_str()).unwrap();
            Ok(TrueType::Primitive(to_true_prim_type(&value, &PrimitiveType::String, &true)?))
        },
        AttrType::Primitive(other) => {
            if let Ok(val) = parse::<Value>(id) {
                Ok(TrueType::Primitive(to_true_prim_type(&val, other, &true)?))
            } else {
                Err(Error::new(ErrorKind::InvalidData, "Invalid value for primary key"))
            }
        },

        // this shouldn't occur, since the model definition should be validated before
        _ => Err(Error::new(ErrorKind::Unsupported, "Arrays for keys aren't allowed"))
    }
}


/*
    parse_model: 
        Parses a valid model in the given path with the given name.

        What happens exactly:
            1. fetch all model definitions via parse_models(),
               this is necessary because of the duplicate filtering
            2. filter the gotten vector of definitions to matches on the given model name (can be only one)
            3. return the matching model, else Error when there are none

    returns:
        A valid model definition with the given model name
        or an Error if there isn't such model defined
*/
pub fn parse_model(model_path: &Path, model_name: &ModelName) -> Result<ModelDefinition>{
    let mut models: Vec<ModelDefinition> = parse_models(model_path)?;
    models.retain(|m| &m.model_name == model_name);
    if models.is_empty() {
        return Err(Error::new(NotFound, format!("model {:?} not found", model_name.0.0)));
    }
    Ok(models.remove(0))
}


/*
    parse_models: 
        Parses all valid models in the given path into a vector.

        What happens exactly:
            1. read the directory structure
            2. iterate through the directory's entries
            3. if an entry can be parsed to a valid model definition,
               push it to the returning vector and memorize the model's name for duplicate checking
            4. remove all models from the returning vector whose name occurs multiple times
            5. return the vector of definitions if there are some, else Error

    returns:
        A vector of valid model definitions, unique by their names
        or an Error if there aren't some
*/
pub fn parse_models(model_path: &Path) -> Result<Vec<ModelDefinition>>{
    // read directory structure
    let model_paths: Result<ReadDir> = read_dir(model_path);
    if model_paths.is_err() {
        return Err(Error::new(NotFound, "No valid models defined"));
    }

    let mut models: Vec<ModelDefinition> = vec!();  // stores the parsed valid models
    let mut model_names: Vec<ModelName> = vec!();  // stores the names of the valid models, just for simpler duplicate checking
    let mut duplicates: Vec<ModelName> = vec!();  // stores all names of duplicate valid models

    // parse the models
    for path in model_paths.unwrap().flatten() {
        // going to parse the file
        // ignore occuring errors, invalid files will be just ignored
        if let Ok(data) = read_to_string(&path.path()) {
            if let Ok(model) = ModelDefinition::try_from(data.as_str()) {
                if model_names.contains(&model.model_name) && !duplicates.contains(&model.model_name) {
                    duplicates.push(model.model_name.clone());
                }
                model_names.push(model.model_name.clone());
                models.push(model);
            }
        }
    }

    // remove duplicates
    for dup in &duplicates {
        models.retain(|m| &m.model_name != dup);
    }

    if models.is_empty() {
        return Err(Error::new(NotFound, "No valid models defined"));
    }
    Ok(models)
}


/*
    parse_record: 
        Parses a given JSON-String to a Record, appropriate to its given model definition.

        What happens exactly:
            1. check if JSON-String is valid JSON and parse it
            2. check if any required attributes are missing,
               keep in mind that the model definition is valid (impl TryFrom),
               so the primary key is required as well
            3. translate the parsed values to their respective type as defined in the model,
               else return Error
            4. check if the Record fits the constraints
            5. return Record

    returns:
        a valid Record for the definition or an Error
        if the given String is not a valid JSON representation of such Record
*/
fn parse_record(json: &str, model: &ModelDefinition) -> Result<Record> {
    let parsed_json = parse::<HashMap<AttrName, Value>>(json);
    
    // check json
    if parsed_json.is_err() {
        return Err(Error::new(InvalidData, "Given JSON-String is not valid JSON"));
    }

    // check for missing required attributes
    for key in &model.required {
        if !parsed_json.as_ref().unwrap().contains_key(key) {
            return Err(Error::new(InvalidData, format!("Missing attribute: {:?}", key.0)));
        };
    }

    let mut record: Record = HashMap::new();

    // convert parsed_json to Record
    for (key, value) in parsed_json.unwrap() {
        let is_required: bool = model.required.contains(&key);
        if let Some(ty) = model.attributes.get(&key) {
            match ty {
                AttrType::Primitive(prim_type) => {
                    match to_true_prim_type(&value, prim_type, &is_required) {
                        Ok(true_prim_value) => record.insert(key, TrueType::Primitive(true_prim_value)),
                        Err(err) => return Err(Error::new(InvalidData, format!("Wrong type of attribute {:?}, {}", key.0, err)))
                    };
                },
                AttrType::Array(arr_type) => {
                    match value.as_array() {
                        Some(arr) => {
                            let mut true_arr: Vec<TruePrimitiveType> = vec!();
                            for val in arr {
                                match to_true_prim_type(val, &arr_type[0], &is_required) {
                                    Ok(true_prim_value) => true_arr.push(true_prim_value),
                                    Err(err) => return Err(Error::new(InvalidData, format!("Wrong type of array attribute {:?}, {}", key.0, err)))
                                };
                            }
                            record.insert(key, TrueType::Array(true_arr));
                        },
                        None => return Err(Error::new(InvalidData, format!("Wrong type of attribute {:?}, expected: Array", key.0)))
                    };
                },
            }
        } else {
            return Err(Error::new(InvalidData, format!("Unknown attribute: {:?}", key.0)));
        }
    }

    check_constraints(&record)?;

    Ok(record)
}

fn check_constraints(record: &Record) -> Result<()> {
    Ok(())
}






#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_record() {
        let valid_input = r#"
            {
                "id": 1,
                "name": "Natural Born Killers",
                "year": 1994,
                "actors": ["Woody Harrelson", "Juliette Lewis"],
                "recommended": true
            }
        "#;
        let expected_record: Record = HashMap::from([
            (AttrName("id".to_string()),TrueType::Primitive(TruePrimitiveType::Integer(1))),
            (AttrName("name".to_string()),TrueType::Primitive(TruePrimitiveType::String("Natural Born Killers".to_string()))),
            (AttrName("year".to_string()),TrueType::Primitive(TruePrimitiveType::Integer(1994))),
            (AttrName("actors".to_string()),TrueType::Array(vec!(TruePrimitiveType::String("Woody Harrelson".to_string()), TruePrimitiveType::String("Juliette Lewis".to_string())))),
            (AttrName("recommended".to_string()),TrueType::Primitive(TruePrimitiveType::Boolean(true)))
        ]);

        let movie_model = ModelDefinition {
            model_name: ModelName(AttrName("movie".to_string())),
            storage_type: StorageType::json,
            attributes: HashMap::from([
                (AttrName("id".to_string()), AttrType::Primitive(PrimitiveType::Integer)),
                (AttrName("name".to_string()), AttrType::Primitive(PrimitiveType::String)),
                (AttrName("year".to_string()), AttrType::Primitive(PrimitiveType::Integer)),
                (AttrName("actors".to_string()), AttrType::Array([PrimitiveType::String])),
                (AttrName("recommended".to_string()), AttrType::Primitive(PrimitiveType::Boolean))
            ]),
            primary_key: AttrName("id".to_string()),
            required: vec!(
                AttrName("id".to_string()),
                AttrName("name".to_string()),
                AttrName("recommended".to_string())
            ),
            constraints: None
        };
        let parsed_record: Record = parse_record(&valid_input.to_string(), &movie_model).unwrap();
        
        assert_eq!(&parsed_record, &expected_record);

        // test errors
        // test String instead of Integer
        let invalid_input = r#"
            {
                "id": "1",
                "name": "Natural Born Killers",
                "year": "1994",
                "actors": ["Woody Harrelson", "Juliette Lewis"],
                "recommended": true
            }
        "#;
        if let Ok(_) = parse_record(invalid_input, &movie_model) {
            assert!(false, "Expected Error for parsing String-Value to Integer");
        }

        // test String instead of Boolean 
        let invalid_input = r#"
            {
                "id": 1,
                "name": "Natural Born Killers",
                "year": 1994,
                "actors": ["Woody Harrelson", "Juliette Lewis"],
                "recommended": "true"
            }
        "#;
        if let Ok(_) = parse_record(invalid_input, &movie_model) {
            assert!(false, "Expected Error for parsing String-Value to Boolean");
        }

        // test Integer instead of String 
        let invalid_input = r#"
            {
                "id": 1,
                "name": 1994,
                "year": 1994,
                "actors": ["Woody Harrelson", "Juliette Lewis"],
                "recommended": true
            }
        "#;
        if let Ok(_) = parse_record(invalid_input, &movie_model) {
            assert!(false, "Expected Error for parsing Integer-Value to String");
        }

        // test Array(Integer) instead of Array(String)
        let invalid_input = r#"
            {
                "id": 1,
                "name": "Natural Born Killers",
                "year": 1994,
                "actors": [1, 2],
                "recommended": true
            }
        "#;
        if let Ok(_) = parse_record(invalid_input, &movie_model) {
            assert!(false, "Expected Error for parsing Array(Integer)-Value to Array(String)");
        }

        // test missing attribute
        let invalid_input = r#"
            {
                "id": 1,
                "year": 1994,
                "actors": ["Woody Harrelson", "Juliette Lewis"],
                "recommended": true
            }
        "#;
        if let Ok(_) = parse_record(invalid_input, &movie_model) {
            assert!(false, "Expected Error for missing required attributes");
        }

        // test null value
        let invalid_input = r#"
            {
                "id": "1",
                "name": null,
                "year": "1994",
                "actors": ["Woody Harrelson", "Juliette Lewis"],
                "recommended": "true"
            }
        "#;
        if let Ok(_) = parse_record(invalid_input, &movie_model) {
            assert!(false, "Expected Error for null-valued required attributes");
        }
        if let Ok(_) = parse_record("invalid json", &movie_model) {
            assert!(false, "Expected Error for parsing invalid JSON input");
        }
    }

    #[test]
    fn test_parse_model() {
        let movie_model = ModelDefinition {
            model_name: ModelName(AttrName("movie".to_string())),
            storage_type: StorageType::json,
            attributes: HashMap::from([
                (AttrName("id".to_string()), AttrType::Primitive(PrimitiveType::Integer)),
                (AttrName("name".to_string()), AttrType::Primitive(PrimitiveType::String)),
                (AttrName("year".to_string()), AttrType::Primitive(PrimitiveType::Integer)),
                (AttrName("actors".to_string()), AttrType::Array([PrimitiveType::String])),
                (AttrName("recommended".to_string()), AttrType::Primitive(PrimitiveType::Boolean))
            ]),
            primary_key: AttrName("id".to_string()),
            required: vec!(
                AttrName("id".to_string()),
                AttrName("name".to_string()),
                AttrName("recommended".to_string())
            ),
            constraints: None
        };

        let expected_result: ModelDefinition = movie_model;
        assert_eq!(&parse_model(Path::new("./testing/server"), &ModelName(AttrName("movie".to_string()))).unwrap(), &expected_result);

        // test errors
        if let Ok(_) = parse_model(Path::new("./testing/server"), &ModelName(AttrName("movie_clone".to_string()))) {
            // test a not existing directory
            assert!(false, "Expected error for parsing a valid model with duplicate model name");
        }
        if let Ok(_) = parse_model(Path::new("./testing/server/not_existing_dir"), &ModelName(AttrName("movie".to_string()))) {
            // test a not existing directory
            assert!(false, "Expected error for not existing models' path");
        }
        if let Ok(_) = parse_model(Path::new("./testing/server/dummy_dir"), &ModelName(AttrName("movie".to_string()))) {
            // test a directory without any valid model definitions
            assert!(false, "Expected error for no matching model definitions");
        }
    }

    #[test]
    fn test_parse_models() {
        let movie_model = ModelDefinition {
            model_name: ModelName(AttrName("movie".to_string())),
            storage_type: StorageType::json,
            attributes: HashMap::from([
                (AttrName("id".to_string()), AttrType::Primitive(PrimitiveType::Integer)),
                (AttrName("name".to_string()), AttrType::Primitive(PrimitiveType::String)),
                (AttrName("year".to_string()), AttrType::Primitive(PrimitiveType::Integer)),
                (AttrName("actors".to_string()), AttrType::Array([PrimitiveType::String])),
                (AttrName("recommended".to_string()), AttrType::Primitive(PrimitiveType::Boolean))
            ]),
            primary_key: AttrName("id".to_string()),
            required: vec!(
                AttrName("id".to_string()),
                AttrName("name".to_string()),
                AttrName("recommended".to_string())
            ),
            constraints: None
        };

        let expected_result: Vec<ModelDefinition> = vec![movie_model];
        assert_eq!(&parse_models(Path::new("./testing/server")).unwrap(), &expected_result);

        // test errors
        if let Ok(_) = parse_models(Path::new("./testing/server/not_existing_dir")) {
            // test a not existing directory
            assert!(false, "Expected error for not existing models' path");
        }
        if let Ok(_) = parse_models(Path::new("./testing/server/dummy_dir")) {
            // test a directory without any valid model definitions
            assert!(false, "Expected error for no existing valid model definitions");
        }
    }
}
