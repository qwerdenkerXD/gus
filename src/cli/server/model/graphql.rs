// use apollo_compiler::hir;
use serde::ser;

// used types
use apollo_compiler::diagnostics::GraphQLError;
use std::collections::HashMap;
use std::sync::Arc;
use super::{
    TruePrimitiveType,
    ModelDefinition,
    PrimitiveType,
    AttrType,
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
    parse_models,
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
    if let Some(args) = crate::cli::get_valid_start_args() {
        let mut type_definitions: String = String::new();
        let mut query_resolvers: Vec<String> = vec!();
        let mut mutation_resolvers: Vec<String> = vec!();
        let mut subscription_resolvers: Vec<String> = vec!();

        let models: Vec<ModelDefinition> = match parse_models(args.modelspath.as_path()) {
            Ok(models) => models,
            Err(_) => return String::new()
        };

        for model in models {
            let pasc_sing_model_name: &str = &model.model_name.pascal().singular().0.0;

            let mut type_def: String = format!("type {} {{", pasc_sing_model_name);
            let mut update_one: String = format!(" updateOne{pasc_sing_model_name}(");
            let mut create_one: String = format!(" addOne{pasc_sing_model_name}(");
            for (attr_name, attr_type) in model.attributes {
                let gql_type = match attr_type {
                    AttrType::Primitive(prim) => to_gql_type(&prim),
                    AttrType::Array(arr) => format!("[{}!]", to_gql_type(&arr[0])),
                };
                let attr: &str = attr_name.0.as_str();
                let attr_ty: &str = gql_type.as_str();
                
                update_one.push_str(format!(" {attr}:{attr_ty}").as_str());
                create_one.push_str(format!(" {attr}:{attr_ty}").as_str());

                if model.primary_key == attr_name {
                    query_resolvers.push(format!(" readOne{pasc_sing_model_name}({attr}:{attr_ty}!):{pasc_sing_model_name}!"));
                    mutation_resolvers.push(format!(" deleteOne{pasc_sing_model_name}({attr}:{attr_ty}!):{pasc_sing_model_name}!"));
                    update_one.push('!');
                }
                if model.required.contains(&attr_name) {
                    create_one.push('!');
                }
                type_def.push_str(format!(" {attr}:{attr_ty}").as_str());
            }
            mutation_resolvers.push(format!("{}):{pasc_sing_model_name}!", update_one.as_str()));
            mutation_resolvers.push(format!("{}):{pasc_sing_model_name}!", create_one.as_str()));
            type_def.push('}');
            type_definitions.push_str(type_def.as_str());
        }

        if !query_resolvers.is_empty() {
            type_definitions.push_str(format!("type Query{{{}}}", query_resolvers.join(" ").as_str()).as_str());
        }
        if !mutation_resolvers.is_empty() {
            type_definitions.push_str(format!("type Mutation{{{}}}", mutation_resolvers.join(" ").as_str()).as_str());
        }
        if !subscription_resolvers.is_empty() {
            type_definitions.push_str(format!("type Subscription{{{}}}", subscription_resolvers.join(" ").as_str()).as_str());
        }

        return type_definitions;
    }
    unreachable!("creating GraphQL schemas is only used for handling HTTP requests, so when the server runs")
}

fn to_gql_type(prim_type: &PrimitiveType) -> String {
    match prim_type {
        PrimitiveType::Integer => "Int".to_string(),
        PrimitiveType::String => "String".to_string(),
        PrimitiveType::Boolean => "Boolean".to_string()
    }
}

pub fn handle_gql_post(body: GraphQLPost) -> GraphQLReturn {
    let mut compiler = Compiler::new(); //.token_limit(...).recursion_limit(...) TODO!
    compiler.add_type_system(&create_schema(), "schema");
    let query_key: FileId = compiler.add_executable(&body.query, "query");
    let validated: Vec<Diagnostic> = compiler.validate();
    if !validated.is_empty() {
        return GraphQLReturn::from(validated.iter().map(|d| d.to_json()).collect::<Errors>());
    }
    let exec_operation: Arc<Operation> = match get_executing_operation(&compiler.db, body.operationName, query_key) {
        Ok(op) => op,
        Err(ret) => return ret
    };

    execute_operation(exec_operation, &compiler.db)
}

fn get_executing_operation(db: &impl HirDatabase, operation_name: Option<String>, db_key: FileId) -> Result<Arc<Operation>, GraphQLReturn> {
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

fn execute_operation(operation: Arc<Operation>, db: &impl HirDatabase) -> GraphQLReturn {
    let mut data = Data::new();
    let mut errors = Errors::new();
    for root_resolver in operation.selection_set().selection() {
        let field: &Arc<Field> = match root_resolver {
            Selection::Field(field) => field,
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!()
        };
        let resolver_name: &str = field.name();
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

        let args: HashMap<&str, TrueType> = HashMap::from_iter(
            field.arguments().iter().map(|arg| (arg.name(), value_to_truetype(arg.value())) )
        );

        let record: Result<Record, std::io::Error> = match prefix {
            "addOne" => create_one(resolver_name.strip_prefix(prefix).unwrap(), serde_json::to_string(&args).unwrap().as_str()),
            "readOne" => {
                let model_name: &str = resolver_name.strip_prefix(prefix).unwrap();
                let id: &str = &args.values().next().unwrap().to_string();
                read_one(model_name, id)
            },
            "updateOne" => {
                let id_attr_name: &str = &field.field_definition(db).unwrap()
                                               .arguments()
                                               .input_values()
                                               .into_iter()
                                               .find(|arg| arg.ty().is_non_null()).unwrap()
                                               .name().to_string();
                update_one(resolver_name.strip_prefix(prefix).unwrap(), &args.get(id_attr_name).unwrap().to_string(), serde_json::to_string(&args).unwrap().as_str())
            },
            "deleteOne" => {
                let model_name: &str = resolver_name.strip_prefix(prefix).unwrap();
                let id: &str = &args.values().next().unwrap().to_string();
                delete_one(model_name, id)
            },
            "" => todo!(),
            _ => unreachable!("there are currently only five root resolver types")
        };

        match record {
            Ok(record) => {
                match resolve_fields(field, record) {
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

fn value_to_truetype(value: &Value) -> TrueType {
    match value {
        Value::Variable(var) => todo!("resolve variable"),
        Value::String { value, .. } => TrueType::Primitive(TruePrimitiveType::String(value.clone())),
        Value::Boolean { value, .. } => TrueType::Primitive(TruePrimitiveType::Boolean(*value)),
        Value::Null { .. } => TrueType::Primitive(TruePrimitiveType::Null(None)),
        Value::List { value, .. } => TrueType::Array(value.iter()
                                                          .map(|val| {
                                                              if let TrueType::Primitive(v) = value_to_truetype(val) {
                                                                  return v;
                                                              }
                                                              unreachable!("arrays store TruePrimitiveType items")
                                                          })
                                                          .collect::<Vec<TruePrimitiveType>>()),
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