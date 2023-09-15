use crate::cli::server::model::StorageTypes;
use std::path::PathBuf;
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

pub fn get_args() -> Cli {
    Cli::parse()
}

pub fn get_start_args() -> Option<StartServer> {
    if let Commands::Start(args) = Cli::parse().command {
        return Some(args);
    }
    None
}

pub fn get_create_model_args() -> Option<CreateModel> {
    if let Commands::CreateModel(args) = Cli::parse().command {
        return Some(args);
    }
    None
}