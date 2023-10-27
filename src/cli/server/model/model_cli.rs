// used types
use dialoguer::console::Style;
use std::collections::HashMap;
use crate::cli::CreateModel;
use super::{
    ModelDefinition,
    StorageType,
    Attributes,
    ModelName,
    AttrType,
    AttrName
};
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
use cruet::string::{
    singularize::to_singular as singularize,
    pluralize::to_plural as pluralize
};
use serde_json::{
    to_string_pretty,
    from_str
};

pub fn create_model(args: CreateModel) {
    let mut attributes: Attributes = HashMap::new();
    let mut primary_key_opts: Vec<String> = vec!();
    let mut required_opts: Vec<String> = vec!();
    let mut required: Vec<AttrName> = vec!();

    // get model name
    let model_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Model Name:")
        .validate_with(ModelNameValidator)
        .interact_text()
        .unwrap();

    // get storage type
    let storage_types: Vec<&str> = vec!(
        "json"
    );
    let storage_type_selection: usize = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Storage Type:")
        .default(0)
        .items(&storage_types)
        .interact()
        .unwrap();
    let storage_type: StorageType = from_str(format!("\"{}\"", storage_types[storage_type_selection]).as_str()).unwrap();

    // define attributes
    loop {
        // get attribute name
        let attr_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Attribute Name:")
            .validate_with(AttrNameValidator)
            .interact_text()
            .unwrap();

        // get attribute type
        let primitives = vec!(
            "String",
            "Integer",
            // "Float",
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

        // get array type it is such and don't add it to the primary key options, since it is not allowed as key
        if types[type_selection] == "Array" {
            let arr_type_selection: usize = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Array Type:")
                .default(0)
                .items(&primitives)
                .interact()
                .unwrap();
            let selected_type = format!("{:?}", primitives[arr_type_selection]);
            let selected_arr_type: AttrType = AttrType::Array([from_str(&selected_type).unwrap()]);
            attributes.insert(AttrName::try_from(attr_name.as_str()).unwrap(), selected_arr_type);
        } else {
            let selected_type = format!("{:?}", types[type_selection]);
            let selected_attr_type: AttrType = from_str(&selected_type).unwrap();
            attributes.insert(AttrName::try_from(attr_name.as_str()).unwrap(), selected_attr_type);

            // don't add attribute names multiple times if they are defined multiple times
            if !primary_key_opts.contains(&attr_name) {
                primary_key_opts.push(attr_name.clone());
            }
        }

        /*
            define constraints here
        */

        // don't add attribute names multiple times if they are defined multiple times
        if !required_opts.contains(&attr_name) {
            required_opts.push(attr_name.clone());
        }

        // don't break up with defining attributes until there are some that are key candidates (so if not Array)
        if !primary_key_opts.is_empty() {
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

    // get primary key
    let id_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Primary Key:")
        .default(0)
        .items(&primary_key_opts)
        .interact()
        .unwrap();
    let primary_key = primary_key_opts[id_selection].to_string();
    
    // automatically set primary key as required
    required.push(AttrName::try_from(primary_key.as_str()).unwrap());

    // don't allow the user to unselect the key as not required
    required_opts.retain(|s| s != &primary_key);

    println!();

    // set up the theme for the multi select because it may happen that the used terminal emulator
    // doesn't show the icon for selected items with the default setting -> ASCII is safe
    let multi_select_theme = ColorfulTheme{
        checked_item_prefix: Style::new().green().apply_to(" [X]".to_string()),
        unchecked_item_prefix: Style::new().red().apply_to(" [ ]".to_string()),
        ..Default::default()
    };

    // get required attributes
    if !required_opts.is_empty() {
        let required_selection = MultiSelect::with_theme(&multi_select_theme)
            .with_prompt("Set required attributes:")
            .items(&required_opts)
            .interact()
            .unwrap();

        for attr_index in required_selection {
            required.push(AttrName::try_from(required_opts[attr_index].as_str()).unwrap());
        }
    }

    // create model definition
    let created_model = ModelDefinition {
        model_name: ModelName(AttrName::try_from(model_name.as_str()).unwrap()),
        storage_type,
        attributes: attributes.clone(),
        primary_key: AttrName::try_from(primary_key.as_str()).unwrap(),
        required,
        constraints: None
    };

    #[cfg(debug_assertions)]
    {
        // this should never fail because the user is not allowed to cause an invalid model
        // so if it really fails, it is a mistake in this dialogue implementation
        assert!(super::validate_model_definition(&created_model).is_ok(), "Invalid model definition");
    }

    // build file path
    let mut modelspath = PathBuf::new();
    modelspath.push(args.modelspath);
    modelspath.push(model_name);
    modelspath.set_extension("json");

    let model_file_path: &Path = modelspath.as_path();

    // try to write the definition to a file, else write it to stdout
    if write(model_file_path, to_string_pretty(&created_model).unwrap()).is_err() {
        println!("{}", &to_string_pretty(&created_model).unwrap());
        eprintln!("unable to write file");
    }
}

struct AttrNameValidator;

impl Validator<String> for AttrNameValidator {
    type Err = String;

    fn validate(&mut self, input: &String) -> Result<(), Self::Err> {
        match AttrName::try_from(input.as_str()) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("{}", err))
        }
    }
}

struct ModelNameValidator;

impl Validator<String> for ModelNameValidator {
    type Err = String;

    fn validate(&mut self, input: &String) -> Result<(), Self::Err> {
        let mut attr_validator = AttrNameValidator;
        attr_validator.validate(input)?;
        if pluralize(input) == singularize(input) {
            return Err("Name has no plural variant".to_string());
        }
        Ok(())
    }
}