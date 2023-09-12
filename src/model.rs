use serde_json::{ Value, from_str as parse, Map as SerdeMap };
use std::fs::{ read_dir, read_to_string, ReadDir };
use std::io::{ Result, ErrorKind::NotFound };
use std::collections::HashMap;

pub fn parse_models() -> Result<Vec<HashMap<String, String>>>{
    let model_paths: ReadDir = read_dir("./models")?;
    let mut models: Vec<HashMap<String, String>> = Vec::new();
    for file in model_paths {
        // going to parse the file
        // ignore occuring errors, invalid files will be just ignored
        if let Ok(path) = file {
            if let Ok(data) = read_to_string(&path.path()) {
                if let Ok(json) = parse::<Value>(&data) {
                    // now converting the parsed object into a HashMap
                    if json.is_object() {
                        let obj: SerdeMap<String, Value> = match json.as_object() {
                            Some(obj) => obj.clone(),
                            None => SerdeMap::new(),
                        };
                        let mut model: HashMap<String, String> = HashMap::new();
                        for (attr, ty) in &obj {
                            // invalid attribute-type-pairs will be ignored
                            if ty.is_string() {
                                match ty.as_str() {
                                    Some(ty) => model.insert(attr.clone(), ty.to_string()),
                                    None => continue,
                                };
                            }
                        }
                        models.push(model);
                    }
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
        let mut movie_model: HashMap<String, String> = HashMap::new();
        movie_model.insert("id".to_string(), "Integer".to_string());
        movie_model.insert("name".to_string(), "String".to_string());
        movie_model.insert("year".to_string(), "Integer".to_string());

        let expected: Vec<HashMap<String, String>> = vec![movie_model];
        assert_eq!(&parse_models().unwrap(), &expected);
    }
}