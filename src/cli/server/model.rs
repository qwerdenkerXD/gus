pub mod types;
mod handler;

// used types
pub use handler::StorageTypes;
use std::collections::HashMap;
use serde_json::Value;
use std::fs::ReadDir;
use std::path::Path;
use crate::cli::index;
use handler::*;
use types::*;
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
use serde_json::{
    from_str as parse,
    to_string as parse_to_string
};
use std::fs::{
    read_to_string,
    read_dir
};

pub fn create_one(model_name: &ModelName, json: &String) -> Result<Record> {
    if let Some(args) = index::get_start_args() {
        let storage_handler = get_handler(&args.storage_type, model_name, &args.modelspath);
        let model = parse_model(args.modelspath.as_path(), model_name);
        if let Err(err) = model {
            return Err(err);
        }
        let record = parse_record(json, &model.unwrap());
        return match record {
            Ok(rec) => Ok(rec),
            Err(err) => Err(err)
        };
    };
    todo!("creating records is currently only possible when the server is running")
}

pub fn parse_model(model_path: &Path, model_name: &ModelName) -> Result<ModelDefinition>{
    let model_paths: Result<ReadDir> = read_dir(model_path);
    if let Err(_) = model_paths {
        return Err(Error::new(NotFound, "No valid models defined"));
    }
    let mut models: Vec<ModelDefinition> = Vec::new();
    for file in model_paths.unwrap() {
        // going to parse the file
        // ignore occuring errors, invalid files will be just ignored
        if let Ok(path) = file {
            if let Ok(data) = read_to_string(&path.path()) {
                if let Ok(model) = ModelDefinition::try_from(&data) {
                    if &model.model_name == model_name {
                        models.push(model);
                    }
                }
            }
        }
    }
    if models.len() > 1 {
        return Err(Error::new(ErrorKind::Other, format!("Duplicate model name {:?}, so not using it", &parse_to_string(model_name).unwrap())));
    }
    if models.len() == 0 {
        return Err(Error::new(NotFound, "No matching valid models defined"));
    }
    Ok(models[0].clone())
}

fn parse_record(json: &String, model: &ModelDefinition) -> Result<Record> {
    let parsed_json = parse::<HashMap<AttrName, Value>>(json);
    if let Err(_) = parsed_json {
        return Err(Error::new(InvalidData, "Given JSON-String is not valid JSON"));
    }

    // check for missing required attributes
    for key in &model.required {
        if !parsed_json.as_ref().unwrap().contains_key(key) {
            return Err(Error::new(InvalidData, ""));
        };
    }

    let mut record: Record = HashMap::new();

    // convert parsed_json to Record
    for (key, value) in parsed_json.unwrap() {
        let is_required: bool = model.required.contains(&key);
        if let Some(ty) = model.attributes.get(&key) {
            match ty {
                AttrType::Primitive(prim_type) => {
                    match to_true_prim_type(&value, &prim_type, &is_required) {
                        Ok(true_prim_value) => record.insert(key, TrueType::Primitive(true_prim_value)),
                        Err(err) => return Err(Error::new(InvalidData, format!("Wrong type of attribute {:?}, {}", key, err)))
                    };
                },
                AttrType::Array(arr_type) => {
                    match value.as_array() {
                        Some(arr) => {
                            let mut true_arr: Vec<TruePrimitiveType> = vec!();
                            for val in arr {
                                match to_true_prim_type(val, &arr_type[0], &is_required) {
                                    Ok(true_prim_value) => true_arr.push(true_prim_value),
                                    Err(err) => return Err(Error::new(InvalidData, format!("Wrong type of array attribute {:?}, {}", key, err)))
                                };
                            }
                            record.insert(key, TrueType::Array(true_arr));
                        },
                        None => return Err(Error::new(InvalidData, format!("Wrong type of attribute {:?}, expected: \"Array\"", key)))
                    };
                },
            }
        } else {
            return Err(Error::new(InvalidData, format!("Unknown attribute: {:?}", key)));
        }
    }

    if let Err(err) = check_constraints(&record) {
        return Err(err);
    }
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
        if let Ok(_) = parse_record(&invalid_input.to_string(), &movie_model) {
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
        if let Ok(_) = parse_record(&invalid_input.to_string(), &movie_model) {
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
        if let Ok(_) = parse_record(&invalid_input.to_string(), &movie_model) {
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
        if let Ok(_) = parse_record(&invalid_input.to_string(), &movie_model) {
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
        if let Ok(_) = parse_record(&invalid_input.to_string(), &movie_model) {
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
        if let Ok(_) = parse_record(&invalid_input.to_string(), &movie_model) {
            assert!(false, "Expected Error for null-valued required attributes");
        }
        if let Ok(_) = parse_record(&"invalid json".to_string(), &movie_model) {
            assert!(false, "Expected Error for parsing invalid JSON input");
        }
    }

    #[test]
    fn test_parse_model() {
        let movie_model = ModelDefinition {
            model_name: ModelName(AttrName("movie".to_string())),
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
        assert_eq!(&parse_model(Path::new("./src/cli/server/test_models"), &ModelName(AttrName("movie".to_string()))).unwrap(), &expected_result);

        // test errors
        if let Ok(_) = parse_model(Path::new("./src/cli/server/test_models"), &ModelName(AttrName("movie_clone".to_string()))) {
            // test a not existing directory
            assert!(false, "Expected error for parsing a valid model with duplicate model name");
        }
        if let Ok(_) = parse_model(Path::new("./src/cli/server/not_existing_dir"), &ModelName(AttrName("movie".to_string()))) {
            // test a not existing directory
            assert!(false, "Expected error for not existing models' path");
        }
        if let Ok(_) = parse_model(Path::new("./src/cli/server/test_models/dummy_dir"), &ModelName(AttrName("movie".to_string()))) {
            // test a directory without any valid model definitions
            assert!(false, "Expected error for no matching model definitions");
        }
    }
}
