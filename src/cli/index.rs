// used types
use super::server::model::StorageTypes;
use std::path::{
    PathBuf,
    Path
};
pub use clap::{
    Error as ClapError,
    error::ErrorKind::ValueValidation,
};
use clap::ValueHint::{
    FilePath,
    DirPath
};

// used traits
use clap::{
    Parser,
    Subcommand,
    CommandFactory
};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Start(StartServer),
    CreateModel(CreateModel),
}

#[derive(Parser, Debug)]
#[clap(name = "start", about = "Starts the webserver")]
pub struct StartServer {
    #[clap(short, long, default_value = "8080", help = "The port to start the webserver on")]
    pub port: u16,
    #[clap(name = "models-path", short, long, default_value = "./", value_name = "DIR", value_hint = DirPath, help = "The path to the model definitions")]
    pub modelspath: PathBuf,
    #[clap(short, long, name = "STORAGE_TYPE", default_value = "json", help = "The path to the model definitions")]
    pub storage_type: StorageTypes
}

#[derive(Parser, Debug)]
#[clap(name = "create-model", about = "An interactive Dialog to create valid model definitions")]
pub struct CreateModel {
    #[clap(name = "models-path", short, long, default_value = "./", value_name = "DIR", value_hint = DirPath, help = "The path to the model definitions")]
    pub modelspath: PathBuf
}

pub fn get_validated_args() -> Result<Cli, ClapError> {
    let mut cli = Cli::parse();
    #[cfg(test)]
    {
        cli = Cli::try_parse_from(vec!["gus", "start", "-m", "./src/cli/server/test_models"]).unwrap();
    }
    validate_args(cli)
}

pub fn get_valid_start_args() -> Option<StartServer> {
    if let Ok(args) = get_validated_args() {
        if let Commands::Start(args) = args.command {
            return Some(args);
        }
    }
    None
}

pub fn get_valid_create_model_args() -> Option<CreateModel> {
    if let Ok(args) = get_validated_args() {
        if let Commands::CreateModel(args) = args.command {
            return Some(args);
        }
    }
    None
}

fn validate_args(mut cli: Cli) -> Result<Cli, ClapError> {
    match cli.command {
        Commands::Start(ref mut start) => {
            if !start.modelspath.as_path().is_dir() {
                return Err(ClapError::raw(ValueValidation, format!("invalid path '{}' for '--models-path <DIR>': '{}' is not a directory", &start.modelspath.display(), &start.modelspath.display())).format(&mut Cli::command()));
            }
        },
        Commands::CreateModel(ref create) => {
            if !create.modelspath.as_path().is_dir() {
                return Err(ClapError::raw(ValueValidation, format!("invalid path '{}' for '--models-path <DIR>': '{}' is not a directory", &create.modelspath.display(), &create.modelspath.display())).format(&mut Cli::command()));
            }
        }
    }
    Ok(cli)
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_Cli_parse() {
        if let Ok(_) = Cli::try_parse_from(vec!["gus", "start", "-p", "-1"]) {
            assert!(false, "Expected Error when passing a negatives to -p");
        }
        if let Ok(_) = Cli::try_parse_from(vec!["gus", "start", "-p", "65536"]) {
            assert!(false, "Expected Error when passing a port greater than maximum of 65535 to -p");
        }
        let mut args: Cli = Cli::try_parse_from(vec!["gus", "start"]).unwrap();
        if let Err(_) = validate_args(args) {
            assert!(false, "Unexpected Error when parsing start args with default values");
        }
        args = Cli::try_parse_from(vec!["gus", "create-model"]).unwrap();
        if let Err(_) = validate_args(args) {
            assert!(false, "Unexpected Error when parsing create-model args with default values");
        }
    }

    #[test]
    fn test_validate_args() {
        let mut args: Cli = Cli::try_parse_from(vec!["gus", "start", "-m", "./not_existing_dir/"]).unwrap();
        if let Ok(_) = validate_args(args) {
            assert!(false, "Expected Error when passing a not existing directory to 'start -m'");
        }
        args = Cli::try_parse_from(vec!["gus", "start", "-m", "./Cargo.toml"]).unwrap();
        if let Ok(_) = validate_args(args) {
            assert!(false, "Expected Error when passing a file to 'start -m'");
        }
        args = Cli::try_parse_from(vec!["gus", "create-model", "-m", "./not_existing_dir/"]).unwrap();
        if let Ok(_) = validate_args(args) {
            assert!(false, "Expected Error when passing a not existing directory to 'create-model -m'");
        }
        args = Cli::try_parse_from(vec!["gus", "create-model", "-m", "./Cargo.toml"]).unwrap();
        if let Ok(_) = validate_args(args) {
            assert!(false, "Expected Error when passing a file to 'create-model -m'");
        }
    }
}