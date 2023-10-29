use serde::ser;

// used types
use std::collections::{
    BTreeMap,
    HashMap
};
use super::{
    ModelDefinition,
    ModelName,
    AttrName,
    TrueType,
    AttrType,
    Record
};
use apollo_parser::cst::{
    OperationDefinition as Operation,
    FragmentDefinition as Fragment,
    CstChildren,
    Selection,
    Argument,
    CstNode,
    Field
};
use serde_derive::{
    Deserialize,
    Serialize
};
use apollo_parser::{
    SyntaxTree,
    Parser
};

// used traits
use std::convert::TryFrom;
use std::convert::From;

// used enums
use apollo_parser::cst::Definition;

// used functions
use serde_json::from_str;
use super::{
    parse_model,
    create_one,
    read_one,
    update_one,
    delete_one
};

type Errors = Vec<String>;
type FieldName = String;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum FieldValue {
    Field(TrueType),
    Resolver(Data)
}

#[derive(Debug)]
pub struct Data {
    map: BTreeMap<FieldName, FieldValue>
}

impl Data {
    fn new() -> Self {
        Self {
            map: BTreeMap::new()
        }
    }
    fn insert(&mut self, key: FieldName, value: FieldValue) {
        let mut key_with_index = format!("{}_", self.map.len());
        key_with_index.push_str(&key);
        self.map.insert(key_with_index, value);
    }
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl ser::Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        use ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.map.len()))?;
        for (k, v) in &self.map {
            let key_without_index = k.split_once('_').unwrap().1;
            map.serialize_entry(key_without_index, v)?;
        }
        map.end()
    }
}


#[derive(Serialize, Debug)]
pub struct GraphQLReturn {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Errors>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Data>
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct GraphQLPost {
    query: String,
    operationName: Option<String>,
    variables: Option<HashMap<String, String>>,
}

impl From<String> for GraphQLPost {
    fn from(value: String) -> Self {
        Self {
            query: value,
            operationName: None,
            variables: None
        }
    }
}

impl TryFrom<&str> for GraphQLPost {
    type Error = GraphQLReturn;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match from_str::<Self>(value) {
            Ok(post) => Ok(post),
            Err(_) => Err(Self::Error {
                errors: Some(vec!("invalid Body".to_string())),
                data: None
            })
        }
    }
}

pub fn handle_gql_post(body: GraphQLPost) -> GraphQLReturn {
    let parser = Parser::new(&body.query); //.token_limit(...).recursion_limit(...) TODO!
    let cst: SyntaxTree = parser.parse();
    if cst.errors().len() > 0 {
        let mut errors: Errors = vec!();
        for err in cst.errors() {
            errors.push(err.message().to_string())
        }
        return GraphQLReturn {
            errors: Some(errors),
            data: None
        };
    }

    let definitions: CstChildren<Definition> = cst.document().definitions();

    let mut fragments = vec!();
    let mut operations = vec!();

    for def in definitions {
        match def {
            Definition::FragmentDefinition(fragment) => fragments.push(fragment),
            Definition::OperationDefinition(op) => operations.push(op),
            _ => ()
        }
    }

    let exec_operation: Operation = match get_executing_operation(operations, body.operationName) {
        Ok(op) => op,
        Err(ret) => return ret
    };

    execute_operation(exec_operation)
}

fn get_executing_operation(mut operations: Vec<Operation>, operation_name: Option<String>) -> Result<Operation, GraphQLReturn> {
    match operations.len() {
        0 => Err(GraphQLReturn {
            errors: Some(Errors::from(["GraphQL Error: Document does not contain any operations".to_string()])),
            data: None
        }),
        1 => Ok(operations.pop().unwrap()),
        _ => {
            if operation_name.is_none() {
                return Err(GraphQLReturn {
                    errors: Some(Errors::from(["GraphQL Error: Document contains more than one operation, missing operation name".to_string()])),
                    data: None
                });
            }
            operations.retain(|o| {
                match o.name() {
                    Some(name) => &name.text() == operation_name.as_ref().unwrap(),
                    None => false
                }
            });
            match operations.pop() {
                Some(o) => Ok(o),
                None => Err(GraphQLReturn {
                    errors: Some(Errors::from([format!("GraphQL Error: Operation with name {} does not exist", operation_name.unwrap().as_str())])),
                    data: None
                })
            }
        }
    }
}

fn execute_operation(operation: Operation) -> GraphQLReturn {
    let mut data = Data::new();
    let mut errors = Errors::new();
    for root_resolver in operation.selection_set().unwrap().selections() {
        let field: Field = match root_resolver {
            Selection::Field(field) => field,
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!()
        };
        let resolver_name: &str = &field.name().unwrap().text();
        let resolved_field_name: String = match field.alias() {
            Some(alias) => alias.name().unwrap().source_string(),
            None => resolver_name.to_string(),
        };
        let prefix: &str;
        if operation.operation_type().is_none() || operation.operation_type().unwrap().query_token().is_some()  {
            if resolver_name.starts_with("readOne") {
                prefix = "readOne";
            } else {
                prefix = "";  // readMany has no prefix because it's the plural variant of the model name
            }
        }
        else if operation.operation_type().unwrap().mutation_token().is_some() {
            if resolver_name.starts_with("addOne") {
                prefix = "addOne";
            } else if resolver_name.starts_with("updateOne") {
                prefix = "updateOne";
            } else if resolver_name.starts_with("deleteOne") {
                prefix = "deleteOne";
            } else {
                errors.push("unable to resolve".to_string());
                continue;
            }
        } else /* if subscription token */ {
            todo!()
        }

        let model_name = ModelName(AttrName(resolver_name.strip_prefix(prefix).unwrap().to_string()));
        if model_name.assert_pascality().is_err() {
            errors.push("unable to resolve".to_string());
            continue;
        }

        let model: ModelDefinition = match parse_model(&model_name) {
            Ok(model) => model,
            Err(_) => {
                errors.push("unable to resolve".to_string());
                continue;
            }
        };

        let mut required_args: Vec<(&AttrName, &AttrType)> = vec!();
        let mut optional_args: Vec<(&AttrName, &AttrType)> = vec!();

        match prefix {
            "readOne" => required_args.push(model.attributes.get_key_value(&model.primary_key).unwrap()),
            "deleteOne" => required_args.push(model.attributes.get_key_value(&model.primary_key).unwrap()),
            "addOne" => {
                for key_value_pair in model.attributes.iter() {
                    if model.required.contains(&key_value_pair.0) {
                        required_args.push(key_value_pair);
                    } else {
                        optional_args.push(key_value_pair);
                    }
                }
            },
            "updateOne" => {
                for key_value_pair in model.attributes.iter() {
                    if key_value_pair.0 == &model.primary_key {
                        required_args.push(key_value_pair);
                    } else {
                        optional_args.push(key_value_pair);
                    }
                }
            },
            ""/*readMany*/ => {
                todo!()
            }
            _ => unreachable!()
        }

        let mut args: CstChildren<Argument> = match get_arguments(&field, required_args, optional_args) {
            Ok(args) => args,
            Err(mut errs) => {
                errors.append(&mut errs);
                continue;
            }
        };

        let record: Result<Record, std::io::Error> = match prefix {
            "readOne" => read_one(&model.model_name.0.0, &args.next().unwrap().value().unwrap().source_string()),
            "deleteOne" => delete_one(&model.model_name.0.0, &args.next().unwrap().value().unwrap().source_string()),
            _ => todo!(),
        };

        match record {
            Ok(record) => {
                match resolve_fields(&field, record) {
                    Ok(resolved) => data.insert(resolved_field_name, FieldValue::Resolver(resolved)),
                    Err(mut errs) => errors.append(&mut errs)
                };
            },
            Err(err) => errors.append(&mut vec!(format!("{}", err)))
        }
    }

    if errors.is_empty() {
        return GraphQLReturn {
            data: Some(data),
            errors: None
        }
    } else if data.is_empty() {
        return GraphQLReturn {
            errors: Some(errors),
            data: None
        }
    }

    GraphQLReturn {
        errors: Some(errors),
        data: Some(data)
    }
}

fn resolve_fields(field: &Field, record: Record) -> Result<Data, Errors> {
    let mut data = Data::new();
    let selections: CstChildren<Selection> = match field.selection_set() {
        Some(selections) => selections.selections(),
        None => return Err(Errors::from(["must have a selection of subfields".to_string()]))
    };
    for sel in selections {
        match sel {
            Selection::Field(field) => {
                if field.selection_set().is_none() {
                    let attr_name = AttrName(field.name().unwrap().source_string());
                    let value: TrueType = match record.get(&attr_name) {
                        Some(val) => val.clone(),
                        None => return Err(Errors::from(["invalid selections".to_string()]))
                    };
                    let resolved_field_name: String = match field.alias() {
                        Some(alias) => alias.name().unwrap().source_string(),
                        None => field.name().unwrap().source_string(),
                    };

                    data.insert(resolved_field_name, FieldValue::Field(value));
                } else {
                    todo!()
                }
            },
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!()
        }
    }

    Ok(data)
}

fn get_arguments(field: &Field, mut required_args: Vec<(&AttrName, &AttrType)>, mut optional_args: Vec<(&AttrName, &AttrType)>) -> Result<CstChildren<Argument>, Errors> {
    let args: CstChildren<Argument> = match field.arguments() {
        Some(args) => args.arguments(),
        None => return Err(Errors::from(["missing arguments".to_string()]))
    };

    for arg in args {
        let name: &str = &arg.name().unwrap().source_string();
        let possible_args: usize = required_args.len() + optional_args.len();
        required_args.retain(|a| a.0.0 != name);
        optional_args.retain(|a| a.0.0 != name);
        if required_args.len() + optional_args.len() == possible_args {
            return Err(Errors::from([format!("unknown argument \"{name}\"")]))
        }
    }

    if !required_args.is_empty() {
        return Err(Errors::from(["missing required argument".to_string()]));
    }

    Ok(field.arguments().unwrap().arguments())
}