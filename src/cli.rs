pub mod index;
mod server;

// used types
use dialoguer::console::Style;
use std::collections::HashMap;
use clap::Parser;
use dialoguer::{
    theme::ColorfulTheme,
    MultiSelect,
    Validator,
    Confirm,
    Select,
    Input
};
use std::path::{
    PathBuf,
    Path
};

// used functions
use std::fs::write;
use serde_json::{
    from_str,
    to_string_pretty
};

pub fn get_args() -> index::Cli {
    index::Cli::parse()
}

pub fn start(args: index::StartServer) {
    let modelspath: &Path = Path::new(&args.modelspath);
    if let Err(_) = server::model::parse_models(modelspath) {
        println!("Warning: No models defined in {}", modelspath.display());
    }
    unimplemented!();
}

pub fn create_model(args: index::CreateModel) {
    if let Ok(exists) = Path::new(&args.modelspath.clone()).try_exists() {
        if !exists {
            eprintln!("The given models' path does not exist");
            return;
        }
    }

    let mut attributes: server::model::types::Attributes = HashMap::new();
    let mut primary_key_opts: Vec<String> = vec!();
    let mut required_opts: Vec<String> = vec!();
    let mut required: Vec<server::model::types::AttrName> = vec!();

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

        let primitives = vec!(
            "String",
            "Integer",
            "Boolean"
        );
        let mut types: Vec<&str> = primitives.clone();
        types.extend(vec!("Array"));
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
                .items(&primitives)
                .interact()
                .unwrap();
            let selected_type = format!("{:?}", primitives[arr_type_selection]);
            let selected_arr_type: server::model::types::AttrType = server::model::types::AttrType::Array([from_str(&selected_type).unwrap()]);
            attributes.insert(server::model::types::AttrName(attr_name.clone()), selected_arr_type);
        } else {
            let selected_type = format!("{:?}", types[type_selection]);
            let selected_attr_type: server::model::types::AttrType = from_str(&selected_type).unwrap();
            attributes.insert(server::model::types::AttrName(attr_name.clone()), selected_attr_type);
            primary_key_opts.push(attr_name.clone());
        }

        required_opts.push(attr_name.clone());

        if primary_key_opts.len() > 0 {
            println!();
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you want to add another attribute?")
                .interact()
                .unwrap()
            {
                break;
            }
        }
        println!();
    }

    // define primary key
    let id_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Primary Key:")
        .default(0)
        .items(&primary_key_opts)
        .interact()
        .unwrap();
    let primary_key = primary_key_opts[id_selection].to_string();
    required.push(server::model::types::AttrName(primary_key.clone()));
    required_opts.retain(|s| s != &primary_key);

    println!();
    let mut multi_select_theme = ColorfulTheme::default();
    multi_select_theme.checked_item_prefix = Style::new().green().apply_to(" [X]".to_string());
    multi_select_theme.unchecked_item_prefix = Style::new().red().apply_to(" [ ]".to_string());

    if required_opts.len() > 0 {
        let required_selection = MultiSelect::with_theme(&multi_select_theme)
            .with_prompt("Set required attributes:")
            .items(&required_opts)
            .interact()
            .unwrap();

        for attr_index in required_selection {
            required.push(server::model::types::AttrName(required_opts[attr_index].to_string()));
        }
    }

    let created_model = server::model::types::ModelDefinition {
        model_name: server::model::types::AttrName(model_name.clone()),
        attributes: attributes.clone(),
        primary_key: server::model::types::AttrName(primary_key),
        required: required
    };

    #[cfg(debug_assertions)]
    {
        assert!(server::model::validate_model_definition(&created_model).is_ok(), "Invalid model definition");
    }

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

impl Validator<String> for JsonAttrValidator {
    type Err = String;

    fn validate(&mut self, input: &String) -> Result<(), Self::Err> {
        match server::model::types::validate_attr_name(input) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("{}", err))
        }
    }
}