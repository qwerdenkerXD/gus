// used types
use std::collections::HashMap;
use super::model::Record;
use apollo_parser::cst::{
    OperationDefinition as Operation,
    FragmentDefinition as Fragment,
    CstChildren,
    Selection
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
    create_one,
    read_one,
    update_one,
    delete_one
};

pub type GraphQLGet = String;
type Data = HashMap<Root, Record>;
type Errors = Vec<String>;
type Root = String;

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
    for root_resolver in query_op.selection_set().unwrap().selections() {
        match root_resolver {
            Selection::Field(field) => {
                let resolver_name: &str = &field.name().unwrap().text();
                if let Some(model_name) = resolver_name.strip_prefix("readOne") {
                    todo!()
                } else {  // must be the pluralized model name for readMany/search
                    todo!()
                }
            },
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!()
        }
    }

    unreachable!()
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