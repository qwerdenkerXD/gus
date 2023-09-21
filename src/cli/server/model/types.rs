// used modules
use serde::de;
use std::fmt;

// used traits
use std::cmp::PartialEq;
use serde::Deserializer;
use std::convert::TryFrom;
use serde_json::Value;
use serde_derive::{
    Deserialize,
    Serialize
};

// used types
use std::collections::HashMap;
use regex::Regex;
use std::io::{
    ErrorKind,
    Result,
    Error
};

// used functions
pub use serde_json::from_str as parse;

pub type Record = HashMap<AttrName, TrueType>;
pub type Attributes = HashMap<AttrName, AttrType>;

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum AttrType {
    Primitive(PrimitiveType),
    Array([PrimitiveType; 1])
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub enum PrimitiveType {
    Integer,
    String,
    Boolean,
    // Float
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Clone, Debug)]
#[serde(untagged)]
pub enum TrueType {
    Primitive(TruePrimitiveType),
    Array(Vec<TruePrimitiveType>)
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Clone, Debug)]
#[serde(untagged)]
pub enum TruePrimitiveType {
    Integer(i64),
    String(String),
    Boolean(bool),
    // Float(f64),
    Null(Option<()>)
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub enum Constraints {
    Integer(IntConstraint),
    String(StrConstraint),
    Boolean(BoolConstraint),
    Float(FloatConstraint)
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct IntConstraint;

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct StrConstraint;

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct BoolConstraint;

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct FloatConstraint;

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct ArrayConstraint;

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct ModelDefinition {
    pub model_name: ModelName,
    pub attributes: Attributes,
    pub primary_key: AttrName,
    pub required: Vec<AttrName>,
    pub constraints: Option<HashMap<AttrName, Constraints>>
}

impl TryFrom<&String> for ModelDefinition {
    type Error = Error;

    fn try_from(json: &String) -> core::result::Result<Self, Self::Error> {
        if let Ok(model) = parse::<ModelDefinition>(&json) {
            validate_model_definition(&model)?;
            Ok(model)
        } else {
            Err(Error::new(ErrorKind::InvalidData, "no valid JSON"))
        }
    }
}


/*
    validate_model_definition: 
        Validates a given model definition if it meets all important conditions.

        What happens exactly:
            1. validate the primary key,
               therefore check if it is defined in the attributes,
               also check if its type is not Array, since this is not allowed
            2. validate the as required defined attributes,
               therefore check if the primary key is required,
               also check if all declared required attributes are actually defined in th attributes

    returns:
        Empty tuple if the model is valid, else Error
*/
pub fn validate_model_definition(definition: &ModelDefinition) -> Result<()> {
    // validate primary key
    if let Some(ty) = definition.attributes.get(&definition.primary_key) {
        if let AttrType::Array(_) = ty {
            return Err(Error::new(ErrorKind::InvalidData, "invalid primary key"));
        }
    }
    else {
        return Err(Error::new(ErrorKind::InvalidData, "invalid primary key"));
    }

    // validate required attributes
    if !definition.required.contains(&definition.primary_key) {
        return Err(Error::new(ErrorKind::InvalidData, "primary key must be required"));
    }
    for attr in &definition.required {
        if !definition.attributes.contains_key(attr) {
            return Err(Error::new(ErrorKind::InvalidData, format!("invalid required attribute {:?}", &attr)));
        }
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Clone, Debug)]
pub struct ModelName(pub AttrName);

// define AttrName with custom Deserializer that validates REST-ful Strings
#[derive(Serialize, Eq, PartialEq, Hash, Clone, Debug)]
pub struct AttrName(pub String);

impl TryFrom<&String> for AttrName {
    type Error = Error;

    fn try_from(s: &String) -> core::result::Result<Self, Self::Error> {
        validate_attr_name(s)?;
        Ok(AttrName(s.clone()))
    }
}

struct AttrNameVisitor;

impl<'de> de::Visitor<'de> for AttrNameVisitor {
    type Value = AttrName;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("alphabetic String, snake_case or PascalCase or camelCase or spinal-case")
    }

    fn visit_str<E>(self, value: &str) -> core::result::Result<AttrName, E>
    where
        E: de::Error,
    {
        match AttrName::try_from(&value.to_string()) {
            Ok(name) => Ok(name),
            Err(err) => Err(de::Error::custom(format!("{}", err))),
        }
    }
}

impl<'de> de::Deserialize<'de> for AttrName {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AttrNameVisitor)
    }
}

fn validate_attr_name(name: &String) -> Result<()> {
    let mut regex: Vec<Regex> = vec!();
    regex.push(Regex::new(r#"^[A-Z][a-zA-Z]*$"#).unwrap());  // PascalCase
    regex.push(Regex::new(r#"^[a-z][a-zA-Z]*$"#).unwrap());  // camelCase
    regex.push(Regex::new(r#"^[a-z]+(_[a-z]+)*$"#).unwrap());  // snake_case
    regex.push(Regex::new(r#"^[a-z]+(-[a-z]+)*$"#).unwrap());  // spinal-case

    for r in regex {
        if r.is_match(name) {
            return Ok(());
        }
    }
    Err(Error::new(ErrorKind::InvalidData, "Attribute name is not alphabetic in camelCase, PascalCase, snake_case or spinal-case"))
}

pub fn to_true_prim_type(value: &Value, model_type: &PrimitiveType, is_required: &bool) -> Result<TruePrimitiveType> {
    if let Some(_) = value.as_null() {
        if *is_required {
            return Err(Error::new(ErrorKind::InvalidData, "it is required, got: null"));
        } else {
            return Ok(TruePrimitiveType::Null(Some(())));
        }
    }
    match model_type {
        PrimitiveType::Integer => {
            match value.as_i64() {
                Some(val) => Ok(TruePrimitiveType::Integer(val)),
                None => Err(Error::new(ErrorKind::InvalidData, "expected: \"Integer\""))
            }
        },
        PrimitiveType::String => {
            match value.as_str() {
                Some(val) => Ok(TruePrimitiveType::String(val.to_string())),
                None => Err(Error::new(ErrorKind::InvalidData, "expected: \"String\""))
            }
        },
        PrimitiveType::Boolean => {
            match value.as_bool() {
                Some(val) => Ok(TruePrimitiveType::Boolean(val)),
                None => Err(Error::new(ErrorKind::InvalidData, "expected: \"Boolean\""))
            }
        },
        // PrimitiveType::Float => {
        //     match value.as_f64() {
        //         Some(val) => Ok(TruePrimitiveType::Float(val)),
        //         None => Err(Error::new(ErrorKind::InvalidData, "expected: \"Float\""))
        //     }
        // },
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_model_definition() {
        // test primary key of type array
        let model = &ModelDefinition {
            model_name: ModelName(AttrName("Test".to_string())),
            primary_key: AttrName("id".to_string()),
            attributes: HashMap::from([
                (AttrName("id".to_string()), AttrType::Array([PrimitiveType::String]))
            ]),
            required: vec!(AttrName("id".to_string())),
            constraints: None
        };
        assert!(validate_model_definition(model).is_err(), "Expected Error for model definitions with Array as primary key type");

        // test not existing primary key attribute
        let model = &ModelDefinition {
            model_name: ModelName(AttrName("Test".to_string())),
            primary_key: AttrName("id".to_string()),
            attributes: HashMap::new(),
            required: vec!(AttrName("id".to_string())),
            constraints: None
        };
        assert!(validate_model_definition(model).is_err(), "Expected Error for model definitions with missing primary key attribute in attributes");

        // test not required primary key
        let model = &ModelDefinition {
            model_name: ModelName(AttrName("Test".to_string())),
            primary_key: AttrName("id".to_string()),
            attributes: HashMap::from([
                (AttrName("id".to_string()), AttrType::Primitive(PrimitiveType::String))
            ]),
            required: vec!(),
            constraints: None
        };
        assert!(validate_model_definition(model).is_err(), "Expected Error for model definitions with not required primary key");

        // test not existing required attribute
        let model = &ModelDefinition {
            model_name: ModelName(AttrName("Test".to_string())),
            primary_key: AttrName("id".to_string()),
            attributes: HashMap::from([
                (AttrName("id".to_string()), AttrType::Primitive(PrimitiveType::String))
            ]),
            required: vec!(
                AttrName("id".to_string()),
                AttrName("iDontExist".to_string())
            ),
            constraints: None
        };
        assert!(validate_model_definition(model).is_err(), "Expected Error for model definitions with not existing required attributes");
    }
}