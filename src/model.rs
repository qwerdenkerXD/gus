use serde_derive::{ Deserialize, Serialize };
use serde_json::{ from_str as parse, to_string_pretty };
use std::fs::{ read_dir, read_to_string, ReadDir };
use std::io::{ Result, ErrorKind, Error };
use std::collections::HashMap;
use std::cmp::PartialEq;
use std::path::Path;

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum AttrType {
    Primitive(PrimitiveType),
    Array(Array)
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub enum PrimitiveType {
    Integer,
    String,
    Boolean
}

pub type Array = [ArrayType; 1];

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum ArrayType {
    Primitive(PrimitiveType),
    Record(Record)
}

pub type Record = HashMap<String, AttrType>;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ModelDefinition {
    pub model_name: String,
    pub attributes: Record,
    pub primary_key: String,
}

pub fn parse_models(model_path: &Path) -> Result<Vec<ModelDefinition>>{
    let model_paths: Result<ReadDir> = read_dir(model_path);
    let mut map: Record = HashMap::new();
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
                        Some(_) => models.push(model),
                        None => println!("Ignored: {:?} because of invalid primary key", &path.path())
                    }
                }
            }
        }
    }
    if models.len() == 0 {
        return Err(Error::new(ErrorKind::NotFound, "No valid models defined"));
    }
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_models() {
        let mut attributes: Record = HashMap::new();
        attributes.insert("id".to_string(), AttrType::Primitive(PrimitiveType::Integer));
        attributes.insert("name".to_string(), AttrType::Primitive(PrimitiveType::String));
        attributes.insert("year".to_string(), AttrType::Primitive(PrimitiveType::Integer));
        attributes.insert("actors".to_string(), AttrType::Array([ArrayType::Primitive(PrimitiveType::String)]));
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