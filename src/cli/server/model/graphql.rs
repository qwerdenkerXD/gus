// use apollo_compiler::hir;
use serde::ser;

// used types
use apollo_compiler::diagnostics::GraphQLError;
use std::collections::HashMap;
use std::sync::Arc;
use super::{
    AttrName,
    TrueType,
    Record
};
use serde_derive::{
    Deserialize,
    Serialize
};
use apollo_compiler::{
    ApolloDiagnostic as Diagnostic,
    ApolloCompiler as Compiler,
    HirDatabase,
    FileId
};
use apollo_compiler::hir::{
    OperationDefinition as Operation,
    FragmentDefinition as Fragment,
    OperationType,
    SelectionSet,
    Selection,
    Field,
    Value
};

// used functions
use serde_json::from_str;
use super::{
    create_one,
    read_one,
    update_one,
    delete_one
};

type Errors = Vec<GraphQLError>;
type FieldName = String;

enum FieldValue {
    Field(TrueType),
    Resolver(Data)
}

pub struct Data {
    map: Vec<(FieldName, FieldValue)>
}

impl Data {
    fn new() -> Self {
        Self {
            map: vec!()
        }
    }
    fn insert(&mut self, key: FieldName, value: FieldValue) {
        self.map.push((key, value));
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
            match v {
                FieldValue::Field(field) => map.serialize_entry(k, field)?,
                FieldValue::Resolver(data) => map.serialize_entry(k, data)?
            }
        }
        map.end()
    }
}


#[derive(Serialize)]
pub struct GraphQLReturn {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Errors>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Data>
}

impl From<&str> for GraphQLReturn {
    fn from(error_message: &str) -> Self {
        Self {
            errors: Some(Errors::from([
                GraphQLError {
                    message: error_message.to_string(),
                    locations: vec!()
                }
            ])),
            data: None
        }
    }
}

impl From<Errors> for GraphQLReturn {
    fn from(errors: Errors) -> Self {
        Self {
            errors: Some(errors),
            data: None
        }
    }
}

impl From<Data> for GraphQLReturn {
    fn from(data: Data) -> Self {
        Self {
            data: Some(data),
            errors: None
        }
    }
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
            Err(_) => Err(Self::Error::from("invalid body"))
        }
    }
}

fn create_schema() -> String {
    "type Query {readOneMovie(id: String): Movie} type Movie {id: String! name: String! actors: [String] recommended: Boolean!}".to_string()
}

pub fn handle_gql_post(body: GraphQLPost) -> GraphQLReturn {
    let mut compiler = Compiler::new(); //.token_limit(...).recursion_limit(...) TODO!
    compiler.add_type_system(&create_schema(), "schema");
    let query_key: FileId = compiler.add_executable(&body.query, "query");
    let validated: Vec<Diagnostic> = compiler.validate();
    if !validated.is_empty() {
        return GraphQLReturn::from(validated.iter().map(|d| d.to_json()).collect::<Errors>());
    }
    let exec_operation: Arc<Operation> = match get_executing_operation(compiler.db, body.operationName, query_key) {
        Ok(op) => op,
        Err(ret) => return ret
    };

    execute_operation(exec_operation)
}

fn get_executing_operation(db: impl HirDatabase, operation_name: Option<String>, db_key: FileId) -> Result<Arc<Operation>, GraphQLReturn> {
    let operations: Arc<Vec<Arc<Operation>>> = db.all_operations();
    match operations.len() {
        0 => Err(GraphQLReturn::from("document does not contain any executable operations")),
        1 => Ok(operations[0].clone()),
        _ => {
            if operation_name.is_none() {
                return Err(GraphQLReturn::from("document contains more than one operation, missing operation name"));
            }
            let name: &str = &operation_name.unwrap();
            match db.find_operation(db_key, Some(name.to_string())) {
                Some(o) => Ok(o),
                None => Err(GraphQLReturn::from(format!("operation with name {} does not exist", name).as_str()))
            }
        }
    }
}

fn execute_operation(operation: Arc<Operation>) -> GraphQLReturn {
    let mut data = Data::new();
    let mut errors = Errors::new();
    for root_resolver in operation.selection_set().selection() {
        let field: &Arc<Field> = match root_resolver {
            Selection::Field(field) => field,
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!()
        };
        let resolver_name: &str = &field.name();
        let prefix: &str = match operation.operation_ty() {
            OperationType::Query => {
                if resolver_name.starts_with("readOne") {
                    "readOne"
                } else {
                    ""  // readMany has no prefix because it's the plural variant of the model name
                }
            },
            OperationType::Mutation => {
                if resolver_name.starts_with("addOne") {
                    "addOne"
                } else if resolver_name.starts_with("updateOne") {
                    "updateOne"
                } else {
                    "deleteOne" // operation is expected to be validated by handle_gql_post
                }
            },
            OperationType::Subscription => todo!(),
        };

        let args: HashMap<&str, String> = HashMap::from_iter(
            field.arguments().iter().map(|arg| (arg.name(), value_to_str(&arg.value())) )
        );

        let record: Result<Record, std::io::Error> = match prefix {
            "readOne" => {
                let model_name: &str = resolver_name.strip_prefix(prefix).unwrap();
                let id: &str = &args.values().next().unwrap();
                read_one(model_name, id)
            },
            "deleteOne" => delete_one(resolver_name.strip_prefix(prefix).unwrap(), &args.values().next().unwrap()),
            _ => todo!(),
        };

        match record {
            Ok(record) => {
                match resolve_fields(&field, record) {
                    Ok(resolved) => data.insert(field.response_name().to_string(), FieldValue::Resolver(resolved)),
                    Err(mut errs) => errors.append(&mut errs)
                };
            },
            Err(err) => errors.append(&mut vec!(GraphQLError {
                message: format!("{}", err),
                locations: vec!()
            }))
        }
    }

    if errors.is_empty() {
        return GraphQLReturn::from(data);
    } else if data.is_empty() {
        return GraphQLReturn::from(errors);
    }

    GraphQLReturn {
        errors: Some(errors),
        data: Some(data)
    }
}

fn value_to_str(value: &Value) -> String {
    match value {
        Value::Variable(var) => var.name().to_string(),
        Value::String { value, .. } => value.to_string(),
        Value::Boolean { value, .. } => value.to_string(),
        Value::Null { .. } => "null".to_string(),
        Value::List { value, .. } => format!("[{}]", value.iter().map(|val| value_to_str(val)).collect::<Vec<String>>().join(", ")),
        _ => todo!()
    }
}

fn resolve_fields(field: &Field, record: Record) -> Result<Data, Errors> {
    let mut data = Data::new();
    for sel in field.selection_set().selection() {
        match sel {
            Selection::Field(sel_field) => {
                if sel_field.selection_set().selection().is_empty() {
                    let attr_name = AttrName(sel_field.name().to_string());
                    let value: TrueType = record.get(&attr_name).unwrap().clone();
                    data.insert(sel_field.response_name().to_string(), FieldValue::Field(value));
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