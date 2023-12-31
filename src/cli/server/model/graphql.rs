// use apollo_compiler::hir;
use serde::ser;

// used types
use apollo_compiler::execution::GraphQLError;
use apollo_compiler::validation::Valid;
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
    ExecutableDocument,
    Parser,
    Schema,
    Node
};
use apollo_compiler::executable::{
    OperationType,
    SelectionSet,
    Operation,
    Selection,
    Field,
    Type
};
use apollo_compiler::schema::{
    FieldDefinition,
    ExtendedType,
    NamedType
};

// used macros
use apollo_compiler::name as named_type;

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
    fn get(&self, key: &FieldName) -> Option<FieldValue> {
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

            let mut type_def: String = format!("type {pasc_sing_model_name} {{");
            let mut update_one: String = format!(" updateOne{pasc_sing_model_name}(");
            let mut create_one: String = format!(" addOne{pasc_sing_model_name}(");

            let mut attributes: Vec<(&AttrName, &AttrType)> = model.attributes.iter().collect();
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
                    AttrType::Array(arr) => format!("[{ty}!]", ty=to_gql_type(&arr[0])),
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
            mutation_resolvers.push(format!("{update_args}):{pasc_sing_model_name}!", update_args=update_one.as_str()));
            mutation_resolvers.push(format!("{create_args}):{pasc_sing_model_name}!", create_args=create_one.as_str()));
            type_def.push('}');
            type_definitions.push_str(type_def.as_str());
        }

        if !query_resolvers.is_empty() {
            type_definitions.push_str(format!("type Query{{{resolvers}}}", resolvers=query_resolvers.join(" ").as_str()).as_str());
        }
        if !mutation_resolvers.is_empty() {
            type_definitions.push_str(format!("type Mutation{{{resolvers}}}", resolvers=mutation_resolvers.join(" ").as_str()).as_str());
        }
        if !subscription_resolvers.is_empty() {
            type_definitions.push_str(format!("type Subscription{{{resolvers}}}", resolvers=subscription_resolvers.join(" ").as_str()).as_str());
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
    let mut parser = Parser::new(); //.token_limit(...).recursion_limit(...) TODO!

    #[cfg(test)]
    parser.parse_schema(create_schema(), "schema").unwrap().validate().unwrap();

    let schema: &Valid<Schema> = &Valid::assume_valid(parser.parse_schema(create_schema(), "schema").unwrap());
    let document: ExecutableDocument = match parser.parse_executable(schema, &body.query, "query") {
        Ok(doc) => doc,
        Err(err) => return GraphQLReturn::from(err.errors.iter().map(|d| d.to_json()).collect::<Errors>()),
    };

    let valid_document: &Valid<ExecutableDocument> = &match document.validate(schema) {
        Ok(valid_doc) => valid_doc,
        Err(err) => return GraphQLReturn::from(err.errors.iter().map(|d| d.to_json()).collect::<Errors>()),
    };

    match get_executing_operation(valid_document, body.operationName) {
        Ok(op) => execute_operation(op, schema, valid_document),
        Err(ret) => ret
    }
}

fn get_executing_operation(document: &Valid<ExecutableDocument>, operation_name: Option<String>) -> Result<&Node<Operation>, GraphQLReturn> {
    let mut operations /* impl Iterator<Item = &'_ Node<Operation>> */ = document.all_operations();

    if operations.next().is_none() {
        return Err(GraphQLReturn::from("document does not contain any executable operations"));
    }
    if operation_name.is_none() && operations.next().is_some() {
        return Err(GraphQLReturn::from("document contains more than one operation, missing operation name"));
    }

    match document.get_operation(operation_name.as_deref()) {
        Ok(o) => Ok(o),
        Err(_) => Err(GraphQLReturn::from(format!("operation with name {name:?} does not exist", name=operation_name.unwrap().as_str()).as_str()))
    }
}

fn execute_operation(operation: &Node<Operation>, schema: &Valid<Schema>, document: &Valid<ExecutableDocument>) -> GraphQLReturn {
    let mut data = Data::new();
    let mut errors = Errors::new();
    for root_resolver in &operation.selection_set.selections {
        let field: &Node<Field> = match root_resolver {
            Selection::Field(field) => field,
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!()
        };
        
        match field.name.as_str() {
            "__schema" => {
                let record = &Data::from(vec![
                    (FieldName::from("types"), resolve_type_system(schema)),
                    (FieldName::from("queryType"), FieldValue::Object(resolve_type_definition(&named_type!("Query"), schema).unwrap())),
                    (FieldName::from("mutationType"), FieldValue::Object(resolve_type_definition(&named_type!("Mutation"), schema).unwrap())),
                    // (FieldName::from("subscriptionType"), FieldValue::Object(resolve_type_definition(named_type!("Subscription"), schema).unwrap())),
                    (FieldName::from("subscriptionType"), FieldValue::Scalar(NULL)),
                    (FieldName::from("directives"), FieldValue::Scalar(TrueType::Array(Some(vec!())))) // directives currently not supported, so ther are none
                ]);
                data.insert(FieldName::from(field.response_key().as_str()), FieldValue::Object(resolve_selection_set_order(&field.selection_set, field.ty(), record, document)));
            },
            "__type" => match resolve_type_definition(&NamedType::new_unchecked(field.arguments[0].value.as_str().unwrap().into()), schema) {
                Some(res) => data.insert(FieldName::from(field.name.as_str()), FieldValue::Object(resolve_selection_set_order(&field.selection_set, field.ty(), &res, document))),
                None => data.insert(FieldName::from(field.name.as_str()), FieldValue::Scalar(NULL))
            },
            "__typename" => data.insert(FieldName::from(field.name.as_str()), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(operation.operation_type.default_type_name().to_string()))))),

            resolver_name => {
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
                        data.insert(FieldName::from(field.response_key().as_str()), FieldValue::Object(resolve_selection_set_order(&field.selection_set, field.ty(), &fields, document)));
                    },
                    Err(err) => errors.append(&mut vec!(GraphQLError {
                        message: err.to_string(),
                        locations: vec!()
                    }))
                }
            }
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

fn resolve_selection_set_order(selection_set: &SelectionSet, resolver_ty: &Type,  field_data: &Data, document: &Valid<ExecutableDocument>) -> Data {
    let mut data = Data::new();
    for sel in &selection_set.selections {
        match sel {
            Selection::Field(sel_field) => {
                match field_data.get(&FieldName::from(sel_field.name.as_str())) {
                    Some(FieldValue::Objects(sub_data)) => {
                        let resolved: Vec<Data> = sub_data.iter().map(|d| resolve_selection_set_order(&sel_field.selection_set, sel_field.ty(), d, document)).collect();
                        data.insert(FieldName::from(sel_field.name.as_str()), FieldValue::Objects(resolved));
                    },
                    Some(FieldValue::Object(sub_data)) => {
                        let resolved: Data = resolve_selection_set_order(&sel_field.selection_set, sel_field.ty(), &sub_data, document);
                        data.insert(FieldName::from(sel_field.name.as_str()), FieldValue::Object(resolved));
                    },
                    Some(scalar) => data.insert(FieldName::from(sel_field.response_key().as_str()), scalar),
                    None => {
                        assert_eq!(sel_field.name.as_str(), "__typename", "Unhandled field \"{field}\" in graphql request", field=sel_field.name.as_str());
                        data.insert(FieldName::from(sel_field.name.as_str()), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(resolver_ty.inner_named_type().to_string())))));
                    }
                }
            },
            Selection::FragmentSpread(frag) => data.append(resolve_selection_set_order(&document.fragments.get(&frag.fragment_name).unwrap().selection_set, resolver_ty, field_data, document)),
            Selection::InlineFragment(frag) => data.append(resolve_selection_set_order(&frag.selection_set, resolver_ty, field_data, document))
        }
    }

    data
}

fn resolve_type_system(schema: &Valid<Schema>) -> FieldValue {
    let mut types: Vec<Data> = vec!();
    for ty_def in schema.types.keys() {
        if let Some(res) = resolve_type_definition(ty_def, schema) {
            types.push(res);
        }
    }

    FieldValue::Objects(types)
}

fn resolve_type_definition(ty_name: &NamedType, schema: &Valid<Schema>) -> Option<Data> {
    if ty_name.as_str().starts_with("__") {
        return None; // don't resolve unnecessarily introspection types
    }
    let mut data = Data::new();

    let ty_def: &ExtendedType = schema.types.get(ty_name)?;
    
    data.insert(FieldName::from("name"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(ty_name.to_string())))));

    match ty_def {
        ExtendedType::Object(def) => {
            data.insert(FieldName::from("kind"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String("OBJECT".to_string())))));
            match &def.description {
                Some(desc) => data.insert(FieldName::from("description"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(desc.to_string()))))),
                None => data.insert(FieldName::from("description"), FieldValue::Scalar(NULL))
            }
            let fields: Vec<Data> = def.fields.values().map(|f| resolve_field_definition(&f.node, schema)).collect();
            data.insert(FieldName::from("fields"), FieldValue::Objects(fields));
        },
        ExtendedType::Scalar(def) => {
            data.insert(FieldName::from("kind"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String("SCALAR".to_string())))));
            match &def.description {
                Some(desc) => data.insert(FieldName::from("description"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(desc.to_string()))))),
                None => data.insert(FieldName::from("description"), FieldValue::Scalar(NULL))
            }
            data.insert(FieldName::from("fields"), FieldValue::Scalar(NULL));
        },
        _ => return None
    }

    data.insert(FieldName::from("ofType"), FieldValue::Scalar(NULL)); // a type has no ofType if it has a TypeDefinition

    // the following fields get default values because they are currently not used
    data.insert(FieldName::from("interfaces"), FieldValue::Scalar(TrueType::Array(Some(vec!()))));
    data.insert(FieldName::from("enumValues"), FieldValue::Scalar(TrueType::Array(Some(vec!())))); // because it affects enums, not used
    data.insert(FieldName::from("possibleTypes"), FieldValue::Scalar(TrueType::Array(Some(vec!())))); // because it affects interfaces
    data.insert(FieldName::from("inputFields"), FieldValue::Scalar(TrueType::Array(Some(vec!())))); // because it affects input types, not used

    Some(data)
}

fn resolve_type(ty: &Type, schema: &Valid<Schema>) -> FieldValue {
    let mut resolved = Data::from(vec![
        (FieldName::from("name"), FieldValue::Scalar(NULL)),
        (FieldName::from("description"), FieldValue::Scalar(NULL)),
        (FieldName::from("fields"), FieldValue::Scalar(TrueType::Array(Some(vec!())))),
        (FieldName::from("interfaces"), FieldValue::Scalar(TrueType::Array(Some(vec!())))),
        (FieldName::from("possibleTypes"), FieldValue::Scalar(TrueType::Array(Some(vec!())))),
        (FieldName::from("enumValues"), FieldValue::Scalar(TrueType::Array(Some(vec!())))),
        (FieldName::from("inputFields"), FieldValue::Scalar(TrueType::Array(Some(vec!()))))
    ]);
    match ty {
        Type::List(ty) => {
            resolved.insert(FieldName::from("kind"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String("LIST".to_string())))));
            resolved.insert(FieldName::from("ofType"), resolve_type(ty, schema));
        },
        Type::NonNullList(ty) => {
            resolved.insert(FieldName::from("kind"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String("NON_NULL".to_string())))));
            resolved.insert(FieldName::from("ofType"), resolve_type(&Type::List(ty.clone()), schema));
        },
        Type::NonNullNamed(name) => {
            resolved.insert(FieldName::from("kind"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String("NON_NULL".to_string())))));
            resolved.insert(FieldName::from("ofType"), resolve_type(&Type::Named(name.clone()), schema));
        },
        Type::Named(name) =>{
            return match resolve_type_definition(name, schema) {
                Some(res) => FieldValue::Object(res),
                None => FieldValue::Scalar(NULL)
            }
        }
    };

    FieldValue::Object(resolved)
}

fn resolve_field_definition(field: &Node<FieldDefinition>, schema: &Valid<Schema>) -> Data {  // __Field
    let mut data = Data::new();
    data.insert(FieldName::from("name"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(field.name.to_string())))));
    match &field.description {
        Some(desc) => data.insert(FieldName::from("description"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(desc.to_string()))))),
        None => data.insert(FieldName::from("description"), FieldValue::Scalar(NULL))
    }

    let args: Vec<Data> = field.arguments.iter().map(|a| {
        let mut data = Data::from(vec![
            (FieldName::from("name"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(a.name.to_string()))))),
            (FieldName::from("type"), resolve_type(&a.ty, schema))
        ]);
        match &a.description {
            Some(desc) => data.insert(FieldName::from("description"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(desc.to_string()))))),
            None => data.insert(FieldName::from("description"), FieldValue::Scalar(NULL))
        }
        match &a.default_value {
            Some(val) => data.insert(FieldName::from("defaultValue"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::String(val.to_string()))))),
            None => data.insert(FieldName::from("defaultValue"), FieldValue::Scalar(NULL))
        }
        data
    }).collect();

    data.insert(FieldName::from("args"), FieldValue::Objects(args));

    data.insert(FieldName::from("type"), resolve_type(&field.ty, schema));
    data.insert(FieldName::from("isDeprecated"), FieldValue::Scalar(TrueType::Primitive(Some(TruePrimitiveType::Boolean(false)))));
    data.insert(FieldName::from("deprecationReason"), FieldValue::Scalar(NULL));

    data
}