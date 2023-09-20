mod index;
mod server;

pub use index::*;

// used types
use dialoguer::console::Style;
use std::collections::HashMap;
use server::model::{
    ModelDefinition,
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
use serde_json::{
    to_string_pretty,
    from_str
};

pub fn run() -> Option<impl futures::Future<Output = Result<(), std::io::Error>>> {
    let cli: Result<Cli, ClapError> = get_validated_args();
    if let Err(err) = cli {
        if let Err(_) = err.print() {
            eprintln!("{}", err.render());
        }
        return None;
    }
    match cli.unwrap().command {
        Commands::Start(args) => return Some(server::start(args.port)),
        Commands::CreateModel(args) => create_model(args)
    }
    None
}

pub fn create_model(args: CreateModel) {
    if let Ok(exists) = Path::new(&args.modelspath.clone()).try_exists() {
        if !exists {
            eprintln!("The given models' path does not exist");
            return;
        }
    }

    let mut attributes: Attributes = HashMap::new();
    let mut primary_key_opts: Vec<String> = vec!();
    let mut required_opts: Vec<String> = vec!();
    let mut required: Vec<AttrName> = vec!();

    // get model name
    let model_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Model Name:")
        .validate_with(AttrValidator)
        .interact_text()
        .unwrap();

    // define attributes
    loop {
        // get attribute name
        let attr_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Attribute Name:")
            .validate_with(AttrValidator)
            .interact_text()
            .unwrap();

        // get attribute type
        let primitives = vec!(
            "String",
            "Integer",
            "Float",
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
            attributes.insert(AttrName::try_from(&attr_name).unwrap(), selected_arr_type);
        } else {
            let selected_type = format!("{:?}", types[type_selection]);
            let selected_attr_type: AttrType = from_str(&selected_type).unwrap();
            attributes.insert(AttrName::try_from(&attr_name).unwrap(), selected_attr_type);

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

    // get primary key
    let id_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Primary Key:")
        .default(0)
        .items(&primary_key_opts)
        .interact()
        .unwrap();
    let primary_key = primary_key_opts[id_selection].to_string();
    
    // automatically set primary key as required
    required.push(AttrName::try_from(&primary_key).unwrap());

    // don't allow the user to unselect the key as not required
    required_opts.retain(|s| s != &primary_key);

    println!();

    // set up the theme for the multi select because it may happen that the used terminal emulator
    // doesn't show the icon for selected items with the default setting -> ASCII is safe
    let mut multi_select_theme = ColorfulTheme::default();
    multi_select_theme.checked_item_prefix = Style::new().green().apply_to(" [X]".to_string());
    multi_select_theme.unchecked_item_prefix = Style::new().red().apply_to(" [ ]".to_string());

    // get required attributes
    if required_opts.len() > 0 {
        let required_selection = MultiSelect::with_theme(&multi_select_theme)
            .with_prompt("Set required attributes:")
            .items(&required_opts)
            .interact()
            .unwrap();

        for attr_index in required_selection {
            required.push(AttrName::try_from(&required_opts[attr_index].to_string()).unwrap());
        }
    }

    // create model definition
    let created_model = ModelDefinition {
        model_name: ModelName(AttrName::try_from(&model_name).unwrap()),
        attributes: attributes.clone(),
        primary_key: AttrName::try_from(&primary_key).unwrap(),
        required: required,
        constraints: None
    };

    #[cfg(debug_assertions)]
    {
        // this should never fail because the user is not allowed to cause an invalid model
        // so if it really fails, it is a mistake in this dialogue implementation
        assert!(server::model::validate_model_definition(&created_model).is_ok(), "Invalid model definition");
    }

    // build file path
    let mut modelspath = PathBuf::new();
    modelspath.push(args.modelspath);
    modelspath.push(model_name);
    modelspath.set_extension("json");

    let model_file_path: &Path = modelspath.as_path();

    // try to write the definition to a file, else write it to stdout
    if let Err(_) = write(model_file_path, &to_string_pretty(&created_model).unwrap()) {
        println!("{}", &to_string_pretty(&created_model).unwrap());
        eprintln!("unable to write file");
        return;
    }
}

struct AttrValidator;

impl Validator<String> for AttrValidator {
    type Err = String;

    fn validate(&mut self, input: &String) -> Result<(), Self::Err> {
        match AttrName::try_from(input) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("{}", err))
        }
    }
}