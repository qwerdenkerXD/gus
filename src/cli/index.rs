use crate::cli::server::model::StorageTypes;
use std::path::PathBuf;
use std::io::{
    ErrorKind,
    Result,
    Error
};
use clap::{
    Parser,
    Subcommand,
    ValueHint
};
use ValueHint::{
    DirPath,
    FilePath
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
    #[clap(short, long, default_value = "./models", value_name = "DIR", value_hint = DirPath, help = "The path to the model definitions")]
    pub modelspath: PathBuf,
    #[clap(short, long, name = "STORAGE_TYPE", default_value = "json", help = "The path to the model definitions")]
    pub storage_type: StorageTypes,
    #[clap(short, long, default_value = "./data.<STORAGE_TYPE>.gus", value_name = "FILE", value_hint = FilePath, help = "The path to the storage file")]
    pub data: PathBuf
}

#[derive(Parser, Debug)]
#[clap(name = "create-model", about = "An interactive Dialog to create valid model definitions")]
pub struct CreateModel {
    #[clap(short, long, default_value = "./models", value_name = "DIR", value_hint = DirPath, help = "The path to the model definitions")]
    pub modelspath: PathBuf
}

pub fn get_validated_args() -> Result<Cli> {
    let cli = Cli::parse();
    #[cfg(test)]
    {
        let cli = Cli::try_parse_from(vec!["gus", "start", "-m", "./src/cli/server/test_models"]).unwrap();
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

fn validate_args(cli: Cli) -> Result<Cli> {
    match &cli.command {
        Commands::Start(start) => {
            if !start.modelspath.as_path().exists() {
                return Err(Error::new(ErrorKind::InvalidInput, "models' path doesn't exist"));
            }
            if !start.data.as_path().exists() {
                return Err(Error::new(ErrorKind::InvalidInput, "storage file's path doesn't exist"));
            }
        },
        Commands::CreateModel(create) => {
            if !create.modelspath.as_path().exists() {
                return Err(Error::new(ErrorKind::InvalidInput, "models' path doesn't exist"));
            }
        }
    }
    Ok(cli)
}