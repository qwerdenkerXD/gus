pub mod types;

// used types
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
    InvalidInput,
    NotFound
};

// used functions
use serde_json::from_str as parse;
use std::fs::{
    read_to_string,
    read_dir
};

pub fn parse_models(model_path: &Path) -> Result<Vec<types::ModelDefinition>>{
    let model_paths: Result<ReadDir> = read_dir(model_path);
    if let Err(_) = model_paths {
        return Err(Error::new(NotFound, "No valid models defined"));
    }
    let mut models: Vec<types::ModelDefinition> = Vec::new();
    for file in model_paths.unwrap() {
        // going to parse the file
        // ignore occuring errors, invalid files will be just ignored
        if let Ok(path) = file {
            if let Ok(data) = read_to_string(&path.path()) {
                if let Ok(model) = parse::<types::ModelDefinition>(&data) {
                    match validate_model_definition(&model) {
                        Ok(_) => models.push(model),
                        Err(err) => println!("Ignored: {:?}, {}", &path.path(), err)
                    }
                } else {
                    println!("Ignored: {:?}, no valid model", &path.path())
                }
            }
        }
    }
    if models.len() == 0 {
        return Err(Error::new(NotFound, "No valid models defined"));
    }
    Ok(models)
}

pub fn validate_model_definition(definition: &types::ModelDefinition) -> Result<()> {
    // validate primary key
    if let Some(ty) = definition.attributes.get(&definition.primary_key) {
        if let types::AttrType::Array(_) = ty {
            return Err(Error::new(InvalidInput, "invalid primary key"));
        }
    }
    else {
        return Err(Error::new(InvalidInput, "invalid primary key"));
    }

    // validate required attributes
    if !definition.required.contains(&definition.primary_key) {
        return Err(Error::new(InvalidInput, "primary key must be required"));
    }
    for attr in &definition.required {
        if !definition.attributes.contains_key(attr) {
            return Err(Error::new(InvalidInput, format!("invalid required attribute {:?}", &attr)));
        }
    }

    Ok(())
}

pub fn parse_record(json: &String, model: &types::ModelDefinition) -> Result<types::Record> {
    let parsed_json = parse::<HashMap<types::AttrName, Value>>(json);
    if let Err(_) = parsed_json {
        return Err(Error::new(InvalidInput, "Given JSON-String is not valid JSON"));
    }

    // check for missing required attributes
    for key in &model.required {
        if !parsed_json.as_ref().unwrap().contains_key(key) {
            return Err(Error::new(InvalidInput, ""));
        };
    }

    let mut record: types::Record = HashMap::new();

    // convert parsed_json to Record
    for (key, value) in parsed_json.unwrap() {
        let is_required: bool = model.required.contains(&key);
        if let Some(ty) = model.attributes.get(&key) {
            match ty {
                types::AttrType::Primitive(prim_type) => {
                    match types::to_true_prim_type(&value, &prim_type, &is_required) {
                        Ok(true_prim_value) => record.insert(key, types::TrueType::Primitive(true_prim_value)),
                        Err(err) => return Err(Error::new(InvalidInput, format!("Wrong type of attribute {:?}, {}", key, err)))
                    };
                },
                types::AttrType::Array(arr_type) => {
                    match value.as_array() {
                        Some(arr) => {
                            let mut true_arr: Vec<types::TruePrimitiveType> = vec!();
                            for val in arr {
                                match types::to_true_prim_type(val, &arr_type[0], &is_required) {
                                    Ok(true_prim_value) => true_arr.push(true_prim_value),
                                    Err(err) => return Err(Error::new(InvalidInput, format!("Wrong type of array attribute {:?}, {}", key, err)))
                                };
                            }
                            record.insert(key, types::TrueType::Array(true_arr));
                        },
                        None => return Err(Error::new(InvalidInput, format!("Wrong type of attribute {:?}, expected: \"Array\"", key)))
                    };
                },
            }
        } else {
            return Err(Error::new(InvalidInput, format!("Unknown attribute: {:?}", key)));
        }
    }

    if let Err(err) = check_constraints(&record) {
        return Err(err);
    }
    Ok(record)
}

fn check_constraints(record: &types::Record) -> Result<()> {
    Ok(())
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_model_definition() {
        // test primary key of type array
        let model = &types::ModelDefinition {
            model_name: types::AttrName("Test".to_string()),
            primary_key: types::AttrName("id".to_string()),
            attributes: HashMap::from([
                (types::AttrName("id".to_string()), types::AttrType::Array([types::PrimitiveType::String]))
            ]),
            required: vec!(types::AttrName("id".to_string())),
            constraints: None
        };
        assert!(validate_model_definition(model).is_err(), "Expected Error for model definitions with Array as primary key type");

        // test not existing primary key attribute
        let model = &types::ModelDefinition {
            model_name: types::AttrName("Test".to_string()),
            primary_key: types::AttrName("id".to_string()),
            attributes: HashMap::new(),
            required: vec!(types::AttrName("id".to_string())),
            constraints: None
        };
        assert!(validate_model_definition(model).is_err(), "Expected Error for model definitions with missing primary key attribute in attributes");

        // test not required primary key
        let model = &types::ModelDefinition {
            model_name: types::AttrName("Test".to_string()),
            primary_key: types::AttrName("id".to_string()),
            attributes: HashMap::from([
                (types::AttrName("id".to_string()), types::AttrType::Primitive(types::PrimitiveType::String))
            ]),
            required: vec!(),
            constraints: None
        };
        assert!(validate_model_definition(model).is_err(), "Expected Error for model definitions with not required primary key");

        // test not existing required attribute
        let model = &types::ModelDefinition {
            model_name: types::AttrName("Test".to_string()),
            primary_key: types::AttrName("id".to_string()),
            attributes: HashMap::from([
                (types::AttrName("id".to_string()), types::AttrType::Primitive(types::PrimitiveType::String))
            ]),
            required: vec!(
                types::AttrName("id".to_string()),
                types::AttrName("iDontExist".to_string())
            ),
            constraints: None
        };
        assert!(validate_model_definition(model).is_err(), "Expected Error for model definitions with not existing required attributes");
    }

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
        let expected_record: types::Record = HashMap::from([
            (types::AttrName("id".to_string()),types::TrueType::Primitive(types::TruePrimitiveType::Integer(1))),
            (types::AttrName("name".to_string()),types::TrueType::Primitive(types::TruePrimitiveType::String("Natural Born Killers".to_string()))),
            (types::AttrName("year".to_string()),types::TrueType::Primitive(types::TruePrimitiveType::Integer(1994))),
            (types::AttrName("actors".to_string()),types::TrueType::Array(vec!(types::TruePrimitiveType::String("Woody Harrelson".to_string()), types::TruePrimitiveType::String("Juliette Lewis".to_string())))),
            (types::AttrName("recommended".to_string()),types::TrueType::Primitive(types::TruePrimitiveType::Boolean(true)))
        ]);

        let movie_model = types::ModelDefinition {
            model_name: types::AttrName("movie".to_string()),
            attributes: HashMap::from([
                (types::AttrName("id".to_string()), types::AttrType::Primitive(types::PrimitiveType::Integer)),
                (types::AttrName("name".to_string()), types::AttrType::Primitive(types::PrimitiveType::String)),
                (types::AttrName("year".to_string()), types::AttrType::Primitive(types::PrimitiveType::Integer)),
                (types::AttrName("actors".to_string()), types::AttrType::Array([types::PrimitiveType::String])),
                (types::AttrName("recommended".to_string()), types::AttrType::Primitive(types::PrimitiveType::Boolean))
            ]),
            primary_key: types::AttrName("id".to_string()),
            required: vec!(
                types::AttrName("id".to_string()),
                types::AttrName("name".to_string()),
                types::AttrName("recommended".to_string())
            ),
            constraints: None
        };
        let parsed_record: types::Record = parse_record(&valid_input.to_string(), &movie_model).unwrap();
        
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
    fn test_parse_models() {
        let movie_model = types::ModelDefinition {
            model_name: types::AttrName("movie".to_string()),
            attributes: HashMap::from([
                (types::AttrName("id".to_string()), types::AttrType::Primitive(types::PrimitiveType::Integer)),
                (types::AttrName("name".to_string()), types::AttrType::Primitive(types::PrimitiveType::String)),
                (types::AttrName("year".to_string()), types::AttrType::Primitive(types::PrimitiveType::Integer)),
                (types::AttrName("actors".to_string()), types::AttrType::Array([types::PrimitiveType::String])),
                (types::AttrName("recommended".to_string()), types::AttrType::Primitive(types::PrimitiveType::Boolean))
            ]),
            primary_key: types::AttrName("id".to_string()),
            required: vec!(
                types::AttrName("id".to_string()),
                types::AttrName("name".to_string()),
                types::AttrName("recommended".to_string())
            ),
            constraints: None
        };

        let expected_result: Vec<types::ModelDefinition> = vec![movie_model];
        assert_eq!(&parse_models(Path::new("./src/cli/server/test_models")).unwrap(), &expected_result);

        // test errors
        if let Ok(_) = parse_models(Path::new("./src/cli/server/not_existing_dir")) {
            // test a not existing directory
            assert!(false, "Expected error for not existing models' path");
        }
        if let Ok(_) = parse_models(Path::new("./src/cli/server/test_models/dummy_dir")) {
            // test a directory without any valid model definitions
            assert!(false, "Expected error for no existing valid model definitions");
        }
    }
}
