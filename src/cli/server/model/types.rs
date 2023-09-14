use serde_derive::{ Deserialize, Serialize };
use std::cmp::PartialEq;
use std::fmt;
use serde::{ Deserializer };
use serde::de;
use regex::Regex;
use std::io::{ Result, Error, ErrorKind };
use std::collections::HashMap;

pub type Record = HashMap<AttrName, TrueType>;
pub type Attributes = HashMap<AttrName, AttrType>;

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
    Array([PrimitiveType; 1])
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub enum PrimitiveType {
    Integer,
    String,
    Boolean
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum TrueType {
    Primitive(TruePrimitiveType),
    Array(Vec<TruePrimitiveType>)
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum TruePrimitiveType {
    Integer(i32),
    String(String),
    Boolean(bool)
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ModelDefinition {
    pub model_name: AttrName,
    pub attributes: Attributes,
    pub primary_key: AttrName,
    pub required: Vec<AttrName>
}

// define AttrName with custom Deserializer that validates REST-ful Strings
#[derive(Serialize, Eq, PartialEq, Hash, Clone, Debug)]
pub struct AttrName(pub String);

struct AttrNameVisitor;

impl<'de> de::Visitor<'de> for AttrNameVisitor {
    type Value = AttrName;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("alphabetic String, snake_case or camelCase or spinal-case")
    }

    fn visit_str<E>(self, value: &str) -> core::result::Result<AttrName, E>
    where
        E: de::Error,
    {
        if validate_attr_name(&value.to_string()).is_ok() {
            Ok(AttrName(value.to_string()))
        } else {
            Err(de::Error::custom("String is not alphabetic in camelCase, snake_case or spinal-case"))
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

pub fn validate_attr_name(name: &String) -> std::io::Result<()> {
    let camel_case: Regex = Regex::new(r#"^[a-z][a-zA-Z]*$"#).unwrap();
    let snake_case: Regex = Regex::new(r#"^[a-z][_a-z]*[a-z]$"#).unwrap();
    let spinal_case: Regex = Regex::new(r#"^[a-z][-a-z]*[a-z]$"#).unwrap();

    if camel_case.is_match(name) || snake_case.is_match(name) || spinal_case.is_match(name) {
        return Ok(());
    }
    Err(Error::new(ErrorKind::InvalidInput, "Attribute name is not alphabetic in camelCase, snake_case or spinal-case"))
}