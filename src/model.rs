use serde_derive::{ Deserialize, Serialize };
use serde_json::{ from_str as parse };
use std::fs::{ read_dir, read_to_string, ReadDir };
use std::io::{ Result, ErrorKind::NotFound };
use std::collections::HashMap;
use std::cmp::PartialEq;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ModelDefinition {
    attributes: HashMap<String, String>,
    primary_key: String
}

pub fn parse_models() -> Result<Vec<ModelDefinition>>{
    let model_paths: ReadDir = read_dir("./models")?;
    let mut models: Vec<ModelDefinition> = Vec::new();
    for file in model_paths {
        // going to parse the file
        // ignore occuring errors, invalid files will be just ignored
        if let Ok(path) = file {
            if let Ok(data) = read_to_string(&path.path()) {
                if let Ok(model) = parse::<ModelDefinition>(&data) {
                    models.push(model);
                }
            }
        }
    }
    if models.len() == 0 {
        return Err(std::io::Error::new(NotFound, "No valid models defined"));
    }
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_models() {
        let mut attributes: HashMap<String, String> = HashMap::new();
        attributes.insert("id".to_string(), "Integer".to_string());
        attributes.insert("name".to_string(), "String".to_string());
        attributes.insert("year".to_string(), "Integer".to_string());

        let movie_model = ModelDefinition {
            attributes: attributes,
            primary_key: "id".to_string()
        };

        let expected: Vec<ModelDefinition> = vec![movie_model];
        assert_eq!(&parse_models().unwrap(), &expected);
    }
}