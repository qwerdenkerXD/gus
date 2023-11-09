// use apollo_compiler::hir;
use serde::ser;

// used types
use std::collections::HashMap;
use super::{
    TruePrimitiveType,
    ModelDefinition,
    PrimitiveType,
    AttrName,
    AttrType,
    TrueType,
    Record,
    NULL
};
use serde_derive::{
    Deserialize,
    Serialize
};
use apollo_compiler::{
    // ApolloDiagnostic as Diagnostic,
    // ApolloCompiler as Compiler,
    ExecutableDocument,
    GraphQLError,
    // HirDatabase,
    Schema,
    // FileId,
    Node
};
use apollo_compiler::executable::{
    OperationType,
    OperationRef,
    SelectionSet,
    Operation,
    Selection,
    Field,
    Value,
    // FieldDefinition,
    // TypeDefinition,
    Type
};

// used functions
use serde_json::{from_str, to_string_pretty};
use super::{
    parse_models,
    create_one,
    read_one,
    update_one,
    delete_one
};

type Errors = Vec<GraphQLError>;
type FieldName = String;

#[derive(Clone)]
enum FieldValue {
    Scalar(TrueType),
    Objects(Vec<Data>),
    Object(Data)
}

#[derive(Clone)]
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
    fn get(&mut self, key: &FieldName) -> Option<FieldValue> {
        let index: usize = self.map.iter().position(|entry| &entry.0 == key)?;
        Some(self.map[index].1.clone())
    }
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    fn append(&mut self, mut other: Data) {
        self.map.append(&mut other.map);
    }
}

impl From<Vec<(FieldName, FieldValue)>> for Data {
    fn from(vec: Vec<(FieldName, FieldValue)>) -> Self {
        Self {
            map: vec
        }
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
                FieldValue::Scalar(field) => map.serialize_entry(k, field)?,
                FieldValue::Object(data) => map.serialize_entry(k, data)?,
                FieldValue::Objects(data) => map.serialize_entry(k, data)?
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

            let mut attributes: Vec<(&AttrName, &AttrType)> = model.attributes.iter().collect::<Vec<(&AttrName, &AttrType)>>();
            attributes.sort_by(|(a, _), (b, _)| {
                if a == &&model.primary_key {
                    std::cmp::Ordering::Less
                } else if b == &&model.primary_key {
                    std::cmp::Ordering::Greater
                } else {
                    a.0.cmp(&b.0)
                }
            });
            for (attr_name, attr_type) in attributes {
                let gql_type = match attr_type {
                    AttrType::Primitive(prim) => to_gql_type(prim),
                    AttrType::Array(arr) => format!("[{}!]", to_gql_type(&arr[0])),
                };
                let attr: &str = attr_name.0.as_str();
                let attr_ty: &str = gql_type.as_str();
                
                update_one.push_str(format!(" {attr}:{attr_ty}").as_str());
                create_one.push_str(format!(" {attr}:{attr_ty}").as_str());

                if &model.primary_key == attr_name {
                    query_resolvers.push(format!(" readOne{pasc_sing_model_name}({attr}:{attr_ty}!):{pasc_sing_model_name}!"));
                    mutation_resolvers.push(format!(" deleteOne{pasc_sing_model_name}({attr}:{attr_ty}!):{pasc_sing_model_name}!"));
                    update_one.push('!');
                }
                type_def.push_str(format!(" {attr}:{attr_ty}").as_str());
                if model.required.contains(attr_name) {
                    create_one.push('!');
                    type_def.push('!');
                }
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
    // let mut compiler = Compiler::new(); //.token_limit(...).recursion_limit(...) TODO!
    let schema = Schema::parse(create_schema(), "schema");
    let document = ExecutableDocument::parse(&schema, &body.query, "query");

    schema.validate().unwrap();

    if let Err(diagnostics) = document.validate(&schema) {
        return GraphQLReturn::from(diagnostics.iter().map(|d| d.to_json()).collect::<Errors>());
    }

    match get_executing_operation(&document, body.operationName) {
        Ok(op) => execute_operation(op),
        Err(ret) => ret
    }
}

fn get_executing_operation(document: &ExecutableDocument, operation_name: Option<String>) -> Result<&Node<Operation>, GraphQLReturn> {
    let mut operations /* impl Iterator<Item = OperationRef<'_>> */ = document.all_operations();

    if operations.next().is_none() {
        return Err(GraphQLReturn::from("document does not contain any executable operations"));
    }
    if operation_name.is_none() && operations.next().is_some() {
        return Err(GraphQLReturn::from("document contains more than one operation, missing operation name"));
    }

    match document.get_operation(operation_name.as_deref()) {
        Ok(o) => Ok(o.definition()),
        Err(_) => Err(GraphQLReturn::from(format!("operation with name {:?} does not exist", operation_name.unwrap().as_str()).as_str()))
    }
}

fn execute_operation(operation: &Node<Operation>) -> GraphQLReturn {
    let mut data = Data::new();
    let mut errors = Errors::new();
    for root_resolver in &operation.selection_set.selections {
        let field: &Node<Field> = match root_resolver {
            Selection::Field(field) => field,
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!()
        };
        // if field.is_introspection() {
        //     match field.name.as_str() {
        //         "__schema" => {
        //             let record = &mut Data::from(vec![
        //                 (FieldName::from("types"), resolve_type_system(db)),
        //                 (FieldName::from("queryType"), FieldValue::Object(resolve_type_definition(&db.find_type_definition_by_name("Query".to_string()).unwrap(), db).unwrap())),
        //                 (FieldName::from("mutationType"), FieldValue::Object(resolve_type_definition(&db.find_type_definition_by_name("Mutation".to_string()).unwrap(), db).unwrap())),
        //                 // (FieldName::from("subscriptionType"), FieldValue::Object(resolve_type_definition(&db.find_type_definition_by_name("Subscription".to_string()).unwrap(), db).unwrap())),
        //                 (FieldName::from("subscriptionType"), FieldValue::Scalar(NULL)),
        //                 (FieldName::from("directives"), FieldValue::Scalar(TrueType::Array(vec!().into()))) // directives currently not supported, so ther are none
        //             ]);
        //             data.insert(FieldName::from(field.response_name()), FieldValue::Object(resolve_selection_set_order(field.selection_set(), &field.ty(db).unwrap(), record, db)));
        //         },
        //         "__type" => match &db.find_type_definition_by_name(field.arguments()[0].value().as_str().unwrap().to_string()) {
        //             Some(def) => match resolve_type_definition(def, db) {
        //                 Some(mut res) => data.insert(FieldName::from(field.name.as_str()), FieldValue::Object(resolve_selection_set_order(field.selection_set(), &field.ty(db).unwrap(), &mut res, db))),
        //                 None => data.insert(FieldName::from(field.name.as_str()), FieldValue::Scalar(NULL))
        //             },
        //             None => data.insert(FieldName::from(field.name.as_str()), FieldValue::Scalar(NULL))
        //         },
                // "__typename" => data.insert(FieldName::from(field.name.as_str()), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(operation.operation_type.to_string()))))),
        //         _ => unreachable!("as of GraphQL documentation are there only __type, __typename and __schema as introspection fields")
        //     }
        //     continue;
        // }
        let resolver_name: &str = field.name.as_str();
        let prefix: &str = match operation.operation_type {
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
            field.arguments.iter().map(|arg| {
                if let Some(arr) = arg.value.as_list() {
                    (arg.name.as_str(), TrueType::Array(Some(arr.iter().map(|a| from_str::<TruePrimitiveType>(&a.to_string()).unwrap()).collect::<Vec<TruePrimitiveType>>())))
                } else {
                    (arg.name.as_str(), from_str::<TrueType>(&arg.value.to_string()).unwrap())
                }
            })
        );

        let record: Result<Record, std::io::Error> = match prefix {
            "addOne" => create_one(resolver_name.strip_prefix(prefix).unwrap(), serde_json::to_string(&args).unwrap().as_str()),
            "readOne" => {
                let model_name: &str = resolver_name.strip_prefix(prefix).unwrap();
                let id: &str = &args.values().next().unwrap().to_string();
                read_one(model_name, id)
            },
            "updateOne" => {
                let id_attr_name: &str = field.arguments[0].name.as_str();
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
                let mut fields = Data::new();
                for (attr_name, value) in record {
                    fields.insert(attr_name.0, FieldValue::Scalar(value));
                }
                data.insert(FieldName::from(field.response_key().as_str()), FieldValue::Object(resolve_selection_set_order(&field.selection_set, field.ty(), &mut fields)));
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

fn resolve_selection_set_order(selection_set: &SelectionSet, resolver_ty: &Type,  field_data: &mut Data) -> Data {
    let mut data = Data::new();
    for sel in &selection_set.selections {
        match sel {
            Selection::Field(sel_field) => {
                match field_data.get(&FieldName::from(sel_field.name.as_str())) {
                    Some(FieldValue::Objects(mut sub_data)) => {
                        let resolved: Vec<Data> = sub_data.iter_mut().map(|d| resolve_selection_set_order(&sel_field.selection_set, sel_field.ty(), d)).collect();
                        data.insert(FieldName::from(sel_field.name.as_str()), FieldValue::Objects(resolved));
                    },
                    Some(FieldValue::Object(mut sub_data)) => {
                        let resolved: Data = resolve_selection_set_order(&sel_field.selection_set, sel_field.ty(), &mut sub_data);
                        data.insert(FieldName::from(sel_field.name.as_str()), FieldValue::Object(resolved));
                    },
                    Some(scalar) => data.insert(FieldName::from(sel_field.response_key().as_str()), scalar),
                    None => {
                        assert_eq!(sel_field.name.as_str(), "__typename", "Unhandled field \"{}\" in graphql request", sel_field.name.as_str());
                        data.insert(FieldName::from(sel_field.name.as_str()), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(resolver_ty.to_string())))));
                    }
                }
            },
            _ => todo!()
            // Selection::FragmentSpread(frag) => data.append(resolve_selection_set_order(frag.fragment(db).unwrap().selection_set(), resolver_ty, field_data, db)),
            // Selection::InlineFragment(frag) => data.append(resolve_selection_set_order(frag.selection_set(), resolver_ty, field_data, db))
        }
    }

    data
}

// fn resolve_type_system(db: &impl HirDatabase) -> FieldValue {
//     let mut types: Vec<Data> = vec!();
//     for ty_def in db.type_system().type_definitions_by_name.values() {
//         if let Some(res) = resolve_type_definition(ty_def, db) {
//             types.push(res);
//         }
//     }

//     FieldValue::Objects(types)
// }

// fn resolve_type_definition(ty_def: &TypeDefinition, db: &impl HirDatabase) -> Option<Data> {
//     let mut data = Data::new();
//     data.insert(FieldName::from("name"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(ty_def.name().to_string())))));

//     match ty_def {
//         TypeDefinition::ObjectTypeDefinition(def) => {
//             data.insert(FieldName::from("kind"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String("OBJECT".to_string())))));
//             if def.is_introspection() {
//                 return None; // don't resolve unnecessarily introspection types, and also avoid stack overflow
//             }
//             match def.description() {
//                 Some(desc) => data.insert(FieldName::from("description"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(desc.to_string()))))),
//                 None => data.insert(FieldName::from("description"), FieldValue::Scalar(NULL))
//             }
//             let fields: Vec<Data> = def.fields().map(|f| resolve_field_definition(f, db)).collect();
//             data.insert(FieldName::from("fields"), FieldValue::Objects(fields));
//         },
//         TypeDefinition::ScalarTypeDefinition(def) => {
//             data.insert(FieldName::from("kind"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String("SCALAR".to_string())))));
//             match def.description() {
//                 Some(desc) => data.insert(FieldName::from("description"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(desc.to_string()))))),
//                 None => data.insert(FieldName::from("description"), FieldValue::Scalar(NULL))
//             }
//             data.insert(FieldName::from("fields"), FieldValue::Scalar(NULL));
//         },
//         _ => return None
//     }

//     data.insert(FieldName::from("ofType"), FieldValue::Scalar(NULL)); // a type has no ofType if it has a TypeDefinition

//     // the following fields get default values because they are currently not used
//     data.insert(FieldName::from("interfaces"), FieldValue::Scalar(TrueType::Array(vec!().into())));
//     data.insert(FieldName::from("enumValues"), FieldValue::Scalar(TrueType::Array(vec!().into()))); // because it affects enums, not used
//     data.insert(FieldName::from("possibleTypes"), FieldValue::Scalar(TrueType::Array(vec!().into()))); // because it affects interfaces
//     data.insert(FieldName::from("inputFields"), FieldValue::Scalar(TrueType::Array(vec!().into()))); // because it affects input types, not used

//     Some(data)
// }

// fn resolve_type(ty: &Type, db: &impl HirDatabase) -> FieldValue {
//     if ty.is_named() {
//         return match resolve_type_definition(&ty.type_def(db).unwrap(), db) {
//             Some(res) => FieldValue::Object(res),
//             None => FieldValue::Scalar(NULL)
//         }
//     }
//     let mut resolved = Data::from(vec![
//         (FieldName::from("name"), FieldValue::Scalar(NULL)),
//         (FieldName::from("description"), FieldValue::Scalar(NULL)),
//         (FieldName::from("fields"), FieldValue::Scalar(TrueType::Array(vec!().into()))),
//         (FieldName::from("interfaces"), FieldValue::Scalar(TrueType::Array(vec!().into()))),
//         (FieldName::from("possibleTypes"), FieldValue::Scalar(TrueType::Array(vec!().into()))),
//         (FieldName::from("enumValues"), FieldValue::Scalar(TrueType::Array(vec!().into()))),
//         (FieldName::from("inputFields"), FieldValue::Scalar(TrueType::Array(vec!().into())))
//     ]);
//     let of_type: &Type = match ty {
//         Type::NonNull { ty, .. } => {
//             resolved.insert(FieldName::from("kind"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String("NON_NULL".to_string())))));
//             ty
//         },
//         Type::List { ty, .. } => {
//             resolved.insert(FieldName::from("kind"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String("LIST".to_string())))));
//             ty
//         },
//         Type::Named { .. } => unreachable!("handled at the beginning of this function")
//     };
//     resolved.insert(FieldName::from("ofType"), resolve_type(of_type, db));

//     FieldValue::Object(resolved)
// }

// fn resolve_field_definition(field: &FieldDefinition, db: &impl HirDatabase) -> Data {  // __Field
//     let mut data = Data::new();
//     data.insert(FieldName::from("name"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(field.name().to_string())))));
//     match field.description() {
//         Some(desc) => data.insert(FieldName::from("description"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(desc.to_string()))))),
//         None => data.insert(FieldName::from("description"), FieldValue::Scalar(NULL))
//     }

//     let args: Vec<Data> = field.arguments().input_values().iter().map(|a| {
//         let mut data = Data::from(vec![
//             (FieldName::from("name"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(a.name().to_string()))))),
//             (FieldName::from("type"), resolve_type(a.ty(), db))
//         ]);
//         match a.description() {
//             Some(desc) => data.insert(FieldName::from("description"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(desc.to_string()))))),
//             None => data.insert(FieldName::from("description"), FieldValue::Scalar(NULL))
//         }
//         match a.default_value() {
//             Some(desc) => data.insert(FieldName::from("defaultValue"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(value_to_truetype(desc).to_string()))))),
//             None => data.insert(FieldName::from("defaultValue"), FieldValue::Scalar(NULL))
//         }
//         data
//     }).collect();

//     data.insert(FieldName::from("args"), FieldValue::Objects(args));

//     data.insert(FieldName::from("type"), resolve_type(field.ty(), db));
//     data.insert(FieldName::from("isDeprecated"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::Boolean(false)))));
//     data.insert(FieldName::from("deprecationReason"), FieldValue::Scalar(NULL));

//     data
// }