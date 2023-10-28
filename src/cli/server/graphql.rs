// used types
use std::collections::{
    BTreeMap,
    HashMap
};
use super::model::{
    ModelDefinition,
    AttrName,
    TrueType,
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
use super::model::{
    parse_model,
    create_one,
    read_one,
    update_one,
    delete_one
};

pub type GraphQLGet = String;
type RootData = BTreeMap<RootName, FieldData>;
type FieldData = BTreeMap<FieldName, FieldValue>;
type Errors = Vec<String>;
type RootName = String;
type FieldName = String;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum FieldValue {
    Field(TrueType),
    Resolver(FieldData)
}



#[derive(Serialize, Debug)]
pub struct GraphQLReturn {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Errors>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<RootData>
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct GraphQLPost {
    query: String,
    operationName: Option<String>,
    variables: Option<HashMap<String, String>>,
}

impl From<GraphQLGet> for GraphQLPost {
    fn from(value: GraphQLGet) -> Self {
        Self {
            query: value.to_string(),
            operationName: None,
            variables: None
        }
    }
}

impl TryFrom<&str> for GraphQLPost {
    type Error = GraphQLReturn;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match from_str::<GraphQLPost>(value) {
            Ok(post) => Ok(post),
            Err(_) => Err(GraphQLReturn {
                errors: Some(vec!("invalid Body".to_string())),
                data: None
            })
        }
    }
}

pub fn handle_post_request(body: GraphQLPost) -> GraphQLReturn {
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

    match exec_operation.operation_type() {
        Some(ty) => {
            if ty.query_token().is_some() {
                handle_query(exec_operation)
            } else if ty.mutation_token().is_some() {
                handle_mutation(exec_operation)
            } else {
                handle_subscription()
            }
        },
        None => handle_query(exec_operation)
    }
}

pub fn handle_get_request(query: GraphQLGet) -> GraphQLReturn {
    handle_post_request(GraphQLPost::from(query))
}

fn get_executing_operation(mut operations: Vec<Operation>, operation_name: Option<String>) -> Result<Operation, GraphQLReturn> {
    match operations.len() {
        0 => Err(GraphQLReturn {
            errors: Some(vec!("GraphQL Error: Document does not contain any operations".to_string())),
            data: None
        }),
        1 => Ok(operations.pop().unwrap()),
        len => {
            if operation_name.is_none() {
                return Err(GraphQLReturn {
                    errors: Some(vec!("GraphQL Error: Document contains more than one operation, missing operation name".to_string())),
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
                    errors: Some(vec!(format!("GraphQL Error: Operation with name {} does not exist", operation_name.unwrap().as_str()))),
                    data: None
                })
            }
        }
    }
}

fn handle_query(query_op: Operation) -> GraphQLReturn {
    let mut data = RootData::new();
    for root_resolver in query_op.selection_set().unwrap().selections() {
        match root_resolver {
            Selection::Field(field) => {
                let resolver_name: &str = &field.name().unwrap().text();
                if resolver_name == "__schema" || resolver_name == "__type" {
                    match resolve_introspection_field(&field) {
                        Ok(resolved) => data.insert(resolver_name.to_string(), resolved),
                        Err(ret) => return ret
                    };
                } else {
                    if let Some(model_name) = resolver_name.strip_prefix("readOne") {
                        let resolved_field_name: String = match field.alias() {
                            Some(alias) => alias.name().unwrap().source_string(),
                            None => resolver_name.to_string(),
                        };

                        let mut args: CstChildren<Argument> = match get_arguments(&field, vec!("id"), None) {  // TODO in model, get_id_attribute(model_name: &str)
                            Ok(args) => args,
                            Err(ret) => return ret
                        };

                        match read_one(model_name, &args.next().unwrap().value().unwrap().source_string()) {
                            Ok(record) => {
                                match resolve_schema_fields(&field, record) {
                                    Ok(resolved) => data.insert(resolved_field_name, resolved),
                                    Err(ret) => return ret
                                };
                            },
                            Err(err) => return GraphQLReturn {
                                errors: Some(vec!(format!("{}", err))),
                                data: None
                            }
                        }
                    } else {
                        todo!("readMany currently not implemented")
                    }
                }
            },
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!()
        }
    }

    GraphQLReturn {
        data: Some(data),
        errors: None
    }
}

fn handle_mutation(mutation_op: Operation) -> GraphQLReturn {
    for root_resolver in mutation_op.selection_set().unwrap().selections() {
        match root_resolver {
            Selection::Field(field) => {
                let resolver_name: &str = &field.name().unwrap().text();
                if let Some(model_name) = resolver_name.strip_prefix("addOne") {
                    todo!()
                } else if let Some(model_name) = resolver_name.strip_prefix("updateOne") {
                    todo!()
                } else {
                    return GraphQLReturn {
                        errors: Some(vec!(format!("Cannot query field \"{}\" on type \"Query\"", field.name().unwrap().text()))),
                        data: None
                    }
                }
            },
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!()
        }
    }

    unreachable!()
}

fn handle_subscription() -> GraphQLReturn {
    todo!()
}

fn resolve_schema_fields(field: &Field, record: Record) -> Result<FieldData, GraphQLReturn> {
    let mut data = FieldData::new();
    match field.selection_set() {
        Some(selections) => {
            for sel in selections.selections() {
                match sel {
                    Selection::Field(field) => {
                        if field.selection_set().is_none() {
                            let attr_name = AttrName(field.name().unwrap().source_string());
                            let value: TrueType = match record.get(&attr_name) {
                                Some(val) => val.clone(),
                                None => return Err(GraphQLReturn {
                                    errors: Some(vec!("invalid selections".to_string())),
                                    data: None
                                })
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
        },
        None => return Err(GraphQLReturn {
            errors: Some(vec!("must have a selection of subfields".to_string())),
            data: None
        })
    }

    Ok(data)
}

fn get_arguments(field: &Field, mut required_args: Vec<&str>, optional_args: Option<Vec<&str>>) -> Result<CstChildren<Argument>, GraphQLReturn> {
    let args: CstChildren<Argument> = match field.arguments() {
        Some(args) => args.arguments(),
        None => return Err(GraphQLReturn {
            errors: Some(vec!("missing arguments".to_string())),
            data: None
        })
    };

    let mut opt_args = optional_args.unwrap_or(vec!());

    for arg in args {
        let name: &str = &arg.name().unwrap().source_string();
        if required_args.contains(&name) {
            required_args.retain(|a| a != &name);
        } else if opt_args.contains(&name) {
            opt_args.retain(|a| a != &name);
        } else {
            return Err(GraphQLReturn {
                errors: Some(vec!(format!("unknown argument \"{name}\""))),
                data: None
            });
        }
    }

    Ok(field.arguments().unwrap().arguments())
}

fn resolve_introspection_field(field: &Field) -> Result<FieldData, GraphQLReturn> {
    todo!()
}