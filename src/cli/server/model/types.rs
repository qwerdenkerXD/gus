// used modules
use serde::de;
use std::fmt;

// used traits
use super::StorageType;
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
use cruet::case::pascal::to_pascal_case as pascalize;
use cruet::case::camel::to_camel_case as camelize;
pub use serde_json::from_str as parse;
use cruet::string::{
    singularize::to_singular as singularize,
    pluralize::to_plural as pluralize
};

pub const NULL: TrueType = TrueType::Primitive(None);
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

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum TrueType {
    Primitive(Option<TruePrimitiveType>),
    Array(Option<Vec<TruePrimitiveType>>)
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum TruePrimitiveType {
    Integer(i64),
    String(String),
    Boolean(bool),
    // Float(f64),
}

impl TrueType {
    pub fn to_string(&self) -> String {
        match self {
            TrueType::Array(Some(arr)) => format!("{:?}", arr.iter().map(|p| TrueType::Primitive(Some(p.clone())).to_string()).collect::<Vec<String>>()),
            TrueType::Primitive(Some(prim)) => match prim {
                TruePrimitiveType::String(string) => string.to_string(),
                TruePrimitiveType::Integer(val) => format!("{}", val),
                TruePrimitiveType::Boolean(val) => format!("{}", val),
            }
            _ => "null".to_string()
        }
    }
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
    pub storage_type: StorageType,
    pub attributes: Attributes,
    pub primary_key: AttrName,
    pub required: Vec<AttrName>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<HashMap<AttrName, Constraints>>
}

impl TryFrom<&str> for ModelDefinition {
    type Error = Error;

    fn try_from(json: &str) -> core::result::Result<Self, Self::Error> {
        if let Ok(model) = parse::<ModelDefinition>(json) {
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
    // validate model name inflection
    if definition.model_name.singular() == definition.model_name.plural() {
        return Err(Error::new(ErrorKind::InvalidData, "Name has no plural variant"));
    }

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
            return Err(Error::new(ErrorKind::InvalidData, format!("invalid required attribute {:?}", &attr.0)));
        }
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Clone, Debug)]
pub struct ModelName(pub AttrName);

impl ModelName {
    pub fn singular(&self) -> Self {
        ModelName(AttrName(singularize(&self.0.0)))
    }
    pub fn assert_singularity(&self) -> Result<()> {
        if self != &self.singular() {
            return Err(Error::new(ErrorKind::InvalidData, "Expected singular model name, got plural variant"));
        }
        Ok(())
    }
    pub fn plural(&self) -> Self {
        ModelName(AttrName(pluralize(&self.0.0)))
    }
    pub fn assert_plurality(&self) -> Result<()> {
        if self != &self.plural() {
            return Err(Error::new(ErrorKind::InvalidData, "Expected plural model name, got singular variant"));
        }
        Ok(())
    }
    pub fn camel(&self) -> Self {
        ModelName(AttrName(camelize(&self.0.0)))
    }
    pub fn pascal(&self) -> Self {
        ModelName(AttrName(pascalize(&self.0.0)))
    }
}

// define AttrName with custom Deserializer that validates REST-ful Strings
#[derive(Serialize, Eq, PartialEq, Hash, Clone, Debug)]
pub struct AttrName(pub String);

impl TryFrom<&str> for AttrName {
    type Error = Error;

    fn try_from(s: &str) -> core::result::Result<Self, Self::Error> {
        validate_attr_name(s)?;
        Ok(AttrName(s.to_string()))
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
        match AttrName::try_from(value) {
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

fn validate_attr_name(name: &str) -> Result<()> {
    let regex: Vec<Regex> = vec!(
        Regex::new(r#"^[A-Z][a-zA-Z]*$"#).unwrap(),  // PascalCase
        Regex::new(r#"^[a-z][a-zA-Z]*$"#).unwrap(),  // camelCase
        Regex::new(r#"^[a-z]+(_[a-z]+)*$"#).unwrap(),  // snake_case
        Regex::new(r#"^[a-z]+(-[a-z]+)*$"#).unwrap()  // spinal-case
    );

    for r in regex {
        if r.is_match(name) {
            return Ok(());
        }
    }
    Err(Error::new(ErrorKind::InvalidData, "Name is not alphabetic in camelCase, PascalCase, snake_case or spinal-case"))
}

pub fn to_true_prim_type(value: &Value, model_type: &PrimitiveType, is_required: bool) -> Result<Option<TruePrimitiveType>> {
    if value.as_null().is_some() {
        if is_required {
            return Err(Error::new(ErrorKind::InvalidData, "it is required, got: null"));
        } else {
            return Ok(None);
        }
    }
    match model_type {
        PrimitiveType::Integer => {
            match value.as_i64() {
                Some(val) => Ok(Some(TruePrimitiveType::Integer(val))),
                None => Err(Error::new(ErrorKind::InvalidData, "expected: Integer"))
            }
        },
        PrimitiveType::String => {
            match value.as_str() {
                Some(val) => Ok(Some(TruePrimitiveType::String(val.to_string()))),
                None => Err(Error::new(ErrorKind::InvalidData, "expected: String"))
            }
        },
        PrimitiveType::Boolean => {
            match value.as_bool() {
                Some(val) => Ok(Some(TruePrimitiveType::Boolean(val))),
                None => Err(Error::new(ErrorKind::InvalidData, "expected: Boolean"))
            }
        },
        // PrimitiveType::Float => {
        //     match value.as_f64() {
        //         Some(val) => Ok(TruePrimitiveType::Float(val)),
        //         None => Err(Error::new(ErrorKind::InvalidData, "expected: Float"))
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
            storage_type: StorageType::json,
            primary_key: AttrName("id".to_string()),
            attributes: Attributes::from([
                (AttrName("id".to_string()), AttrType::Array([PrimitiveType::String]))
            ]),
            required: vec!(AttrName("id".to_string())),
            constraints: None
        };
        assert!(validate_model_definition(model).is_err(), "Expected Error for model definitions with Array as primary key type");

        // test not existing primary key attribute
        let model = &ModelDefinition {
            model_name: ModelName(AttrName("Test".to_string())),
            storage_type: StorageType::json,
            primary_key: AttrName("id".to_string()),
            attributes: Attributes::new(),
            required: vec!(AttrName("id".to_string())),
            constraints: None
        };
        assert!(validate_model_definition(model).is_err(), "Expected Error for model definitions with missing primary key attribute in attributes");

        // test not required primary key
        let model = &ModelDefinition {
            model_name: ModelName(AttrName("Test".to_string())),
            storage_type: StorageType::json,
            primary_key: AttrName("id".to_string()),
            attributes: Attributes::from([
                (AttrName("id".to_string()), AttrType::Primitive(PrimitiveType::String))
            ]),
            required: vec!(),
            constraints: None
        };
        assert!(validate_model_definition(model).is_err(), "Expected Error for model definitions with not required primary key");

        // test not existing required attribute
        let model = &ModelDefinition {
            model_name: ModelName(AttrName("Test".to_string())),
            storage_type: StorageType::json,
            primary_key: AttrName("id".to_string()),
            attributes: Attributes::from([
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