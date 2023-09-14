mod cli;
mod model;

use clap::Parser;
use dialoguer::{ Select, theme::ColorfulTheme, Input, Validator, Confirm };
use regex::Regex;
use std::fmt::{ Display, Formatter, Result as FmtResult };
use std::collections::HashMap;
use serde_json::{ from_str, to_string_pretty };
use std::fs::{ write };
use std::path::{ Path, PathBuf };

fn main() {
    let args = cli::Cli::parse();
    match args.command {
        cli::Commands::Start(cmd) => start(cmd),
        cli::Commands::CreateModel(cmd) => create_model(cmd)
    }
}

fn start(args: cli::StartServer) {
    let modelspath: &Path = Path::new(&args.modelspath);
    if let Err(_) = model::parse_models(modelspath) {
        println!("Warning: No models defined in {}", modelspath.display());
    }
}

fn create_model(args: cli::CreateModel) {
    if let Ok(exists) = Path::new(&args.modelspath.clone()).try_exists() {
        if !exists {
            eprintln!("The given models' path does not exist");
            return;
        }
    }

    let mut attributes: model::Attributes = HashMap::new();
    let mut attr_names: Vec<String> = vec!();

    let model_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Model Name:")
        .validate_with(JsonAttrValidator)
        .interact_text()
        .unwrap();

    // define attributes
    loop {
        let attr_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Attribute Name:")
            .validate_with(JsonAttrValidator)
            .interact_text()
            .unwrap();

        let primitives = &[
            "String",
            "Integer",
            "Boolean"
        ];
        let mut types: Vec<&str> = vec!("Array");
        types.extend(primitives.clone());
        let type_selection: usize = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Data Type:")
            .default(0)
            .items(&types)
            .interact()
            .unwrap();
        if types[type_selection] == "Array" {
            let arr_type_selection: usize = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Array Type:")
                .default(0)
                .items(primitives)
                .interact()
                .unwrap();
            let selected_type = format!("{:?}", primitives[arr_type_selection]);
            let selected_arr_type: model::AttrType = model::AttrType::Array([from_str(&selected_type).unwrap()]);
            attributes.insert(attr_name.clone(), selected_arr_type);
        } else {
            let selected_type = format!("{:?}", types[type_selection]);
            let selected_attr_type: model::AttrType = from_str(&selected_type).unwrap();
            attributes.insert(attr_name.clone(), selected_attr_type);
            attr_names.push(attr_name);
        }

        println!();
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to add another attribute?")
            .interact()
            .unwrap()
        {
            break;
        }
        println!();
    }

    // define primary key
    let id_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Primary Key:")
        .default(0)
        .items(&attr_names)
        .interact()
        .unwrap();
    let primary_key = attr_names[id_selection].to_string();

    let created_model = model::ModelDefinition {
        model_name: model_name.clone(),
        attributes: attributes.clone(),
        primary_key: primary_key
    };

    let mut modelspath: PathBuf = PathBuf::new();
    modelspath.push(args.modelspath);
    modelspath.push(model_name);
    modelspath.set_extension("json");

    let model_file_path: &Path = modelspath.as_path();

    if let Err(_) = write(model_file_path, &to_string_pretty(&created_model).unwrap()) {
        println!("{}", &to_string_pretty(&created_model).unwrap());
        eprintln!("unable to write file");
        return;
    }
}

struct JsonAttrValidator;

#[derive(Debug)]
struct ValidationError(String);

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Validator<String> for JsonAttrValidator {
    type Err = ValidationError;

    fn validate(&mut self, input: &String) -> Result<(), Self::Err> {
        let re = Regex::new(r#"^[a-zA-Z_$][a-zA-Z_$0-9]*$"#).unwrap();
        if re.is_match(input) {
            Ok(())
        } else {
            Err(ValidationError("No valid JSON".to_string()))
        }
    }
}