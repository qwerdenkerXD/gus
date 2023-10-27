// used types
use std::collections::HashMap;
use super::model::Record;
use apollo_parser::cst::{
    FragmentDefinition as Fragment,
    OperationDefinition as Query,
    CstChildren
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

    #[serde(skip_serializing_if = "Option::is_none")]
    operationName: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
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
    let parser = Parser::new(&body.query);
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
    let mut queries = vec!();

    for def in definitions {
        match def {
            Definition::FragmentDefinition(fragment) => fragments.push(fragment),
            Definition::OperationDefinition(query) => queries.push(query),
            _ => ()
        }
    }

    handle_queries(Queries{
        fragments,
        queries
    })
}

pub fn handle_get_request(query: GraphQLGet) -> GraphQLReturn {
    handle_post_request(GraphQLPost::from(query))
}

struct Queries {
    fragments: Vec<Fragment>,
    queries: Vec<Query>
}

fn handle_queries(queries: Queries) -> GraphQLReturn {
    todo!()
}
