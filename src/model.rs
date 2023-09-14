use serde_derive::{ Deserialize, Serialize };
use serde_json::{ from_str as parse, Value };
use std::fs::{ read_dir, read_to_string, ReadDir };
use std::io::{ Result, ErrorKind, Error };
use std::collections::HashMap;
use std::cmp::PartialEq;
use std::path::Path;

pub trait ModelHandler {
    fn create_one(record: Record) -> Result<Record>;
    fn read_one(id: PrimitiveType) -> Result<Record>;
    fn update_one(id: PrimitiveType, record: Record) -> Result<Record>;
    fn delete_one(id: PrimitiveType) -> Result<Record>;
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum AttrType {
    Primitive(PrimitiveType),
    Array(ArrayType)
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub enum PrimitiveType {
    Integer,
    String,
    Boolean
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum OriginalType {
    Primitive(OriginalPrimitive),
    Array(OriginalArray)
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum OriginalPrimitive {
    Integer(i32),
    String(String),
    Boolean(bool)
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum OriginalArray {
    Array(Vec<OriginalPrimitive>)
}

pub type ArrayType = [PrimitiveType; 1];

pub type Record = HashMap<String, OriginalType>;

pub type Attributes = HashMap<String, AttrType>;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ModelDefinition {
    pub model_name: String,
    pub attributes: Attributes,
    pub primary_key: String
}

pub fn parse_models(model_path: &Path) -> Result<Vec<ModelDefinition>>{
    let model_paths: Result<ReadDir> = read_dir(model_path);
    if let Err(_) = model_paths {
        return Err(Error::new(ErrorKind::NotFound, "No valid models defined"));
    }
    let mut models: Vec<ModelDefinition> = Vec::new();
    for file in model_paths.unwrap() {
        // going to parse the file
        // ignore occuring errors, invalid files will be just ignored
        if let Ok(path) = file {
            if let Ok(data) = read_to_string(&path.path()) {
                if let Ok(model) = parse::<ModelDefinition>(&data) {
                    // now validate primary key
                    match model.attributes.get(&model.primary_key) {
                        Some(ty) => {
                            if let AttrType::Primitive(_) = ty {
                                 models.push(model);
                            } else {
                                println!("Ignored: {:?} because of invalid primary key", &path.path());
                            }
                        },
                        None => println!("Ignored: {:?} because of invalid primary key", &path.path())
                    }
                } else {
                    println!("Ignored: {:?}, no valid model", &path.path())
                }
            }
        }
    }
    if models.len() == 0 {
        return Err(Error::new(ErrorKind::NotFound, "No valid models defined"));
    }
    Ok(models)
}

fn parse_record(json: &str, model: &ModelDefinition) -> Result<Record> {
    let parsed_json = parse::<HashMap<String, Value>>(json);
    if let Ok(parsed) = parsed_json {
        for (key, value) in parsed {
            if model.attributes.get(&key).is_some() {
                unimplemented!(); // unimplemented
            } else {
                return Err(Error::new(ErrorKind::InvalidInput, "Given JSON-String doesn't match model definition"));
            }
        }
    } else {
        return Err(Error::new(ErrorKind::InvalidInput, "Given JSON-String is not valid JSON"));
    }
    Ok(HashMap::new())
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
        let mut expected_record: Record = HashMap::new();
        expected_record.insert("id".to_string(),OriginalType::Primitive(OriginalPrimitive::Integer(1)));
        expected_record.insert("name".to_string(),OriginalType::Primitive(OriginalPrimitive::String("Natural Born Killers".to_string())));
        expected_record.insert("year".to_string(),OriginalType::Primitive(OriginalPrimitive::Integer(1994)));
        expected_record.insert("actors".to_string(),OriginalType::Array(OriginalArray::Array(vec!(OriginalPrimitive::String("Woody Harrelson".to_string()), OriginalPrimitive::String("Juliette Lewis".to_string())))));
        expected_record.insert("recommended".to_string(),OriginalType::Primitive(OriginalPrimitive::Boolean(true)));

        let mut attributes: Attributes = HashMap::new();
        attributes.insert("id".to_string(), AttrType::Primitive(PrimitiveType::Integer));
        attributes.insert("name".to_string(), AttrType::Primitive(PrimitiveType::String));
        attributes.insert("year".to_string(), AttrType::Primitive(PrimitiveType::Integer));
        attributes.insert("actors".to_string(), AttrType::Array([PrimitiveType::String]));
        attributes.insert("recommended".to_string(), AttrType::Primitive(PrimitiveType::Boolean));

        let movie_model = ModelDefinition {
            model_name: "movie".to_string(),
            attributes: attributes,
            primary_key: "id".to_string()
        };
        let parsed_record: Record = parse_record(&valid_input, &movie_model).unwrap();
        
        assert_eq!(&parsed_record, &expected_record);

        // test errors
        let invalid_input = r#"
            {
                "id": "1",
                "name": "Natural Born Killers",
                "year": "1994",
                "actors": ["Woody Harrelson", "Juliette Lewis"],
                "recommended": "true"
            }
        "#;
        if let Ok(_) = parse_record(&invalid_input, &movie_model) {
            assert!(false, "Expected Error for parsing invalid types");;
        }
        if let Ok(_) = parse_record("invalid json", &movie_model) {
            assert!(false, "Expected Error for parsing invalid JSON input");;
        }
    }

    #[test]
    fn test_parse_models() {
        let mut attributes: Attributes = HashMap::new();
        attributes.insert("id".to_string(), AttrType::Primitive(PrimitiveType::Integer));
        attributes.insert("name".to_string(), AttrType::Primitive(PrimitiveType::String));
        attributes.insert("year".to_string(), AttrType::Primitive(PrimitiveType::Integer));
        attributes.insert("actors".to_string(), AttrType::Array([PrimitiveType::String]));
        attributes.insert("recommended".to_string(), AttrType::Primitive(PrimitiveType::Boolean));

        let movie_model = ModelDefinition {
            model_name: "movie".to_string(),
            attributes: attributes,
            primary_key: "id".to_string()
        };

        let expected_result: Vec<ModelDefinition> = vec![movie_model];
        assert_eq!(&parse_models(Path::new("./models")).unwrap(), &expected_result);

        // test errors
        if let Ok(_) = parse_models(Path::new("./not_existing_dir")) {
            // test a not existing directory
            assert!(false, "Expected error for not existing models' path");
        }
        if let Ok(_) = parse_models(Path::new("./models/dummy_dir")) {
            // test a directory without any valid model definitions
            assert!(false, "Expected error for no existing valid model definitions");
        }
    }
}