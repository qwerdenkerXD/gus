// used types
use std::path::PathBuf;
use std::net::Ipv4Addr;
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
    ConfigureStorages(ConfigureStorages)
}

#[derive(Parser, Debug)]
#[clap(name = "start", about = "Starts the webserver")]
pub struct StartServer {
    #[clap(short, long, default_value = "127.0.0.1", value_name = "IPv4", help = "The binding adress to start the webserver on")]
    pub bind: Ipv4Addr,
    #[clap(short, long, default_value = "8080", help = "The port to start the webserver on")]
    pub port: u16,
    #[clap(name = "models-path", short, long, default_value = "./", value_name = "DIR", value_hint = DirPath, help = "The path to the model definitions")]
    pub modelspath: PathBuf,
    #[clap(name = "storage-definitions", short, long, value_name = "FILE", value_hint = FilePath, help = "The path to the storage definitions' file")]
    pub storage_definitions: Option<PathBuf>
}

#[derive(Parser, Debug)]
#[clap(name = "create-model", about = "An interactive Dialog to create valid model definitions")]
pub struct CreateModel {
    #[clap(name = "models-path", short, long, default_value = "./", value_name = "DIR", value_hint = DirPath, help = "The path to the model definitions")]
    pub modelspath: PathBuf
}

#[derive(Parser, Debug)]
#[clap(name = "configure-storages", about = "An interactive Dialog to configure storage types properly")]
pub struct ConfigureStorages {
    #[clap(name = "storage-definitions", short, long, default_value = "./storages.json", value_name = "FILE", value_hint = FilePath, help = "The path to the storage definitions' file")]
    pub storage_definitions: PathBuf
}

pub fn get_validated_args() -> Result<Cli, ClapError> {
    let cli = Cli::parse();

    validate_args(cli)
}

pub fn get_valid_start_args() -> Option<StartServer> {
    #[cfg(not(test))]
    {
        if let Ok(args) = get_validated_args() {
            if let Commands::Start(args) = args.command {
                return Some(args);
            }
        }

        None
    }

    #[cfg(test)]
    {
        Some(StartServer::try_parse_from(vec!["start", "-m", "./testing/server", "-s", "./testing/server/storages.json"]).unwrap())
    }
}

fn validate_args(cli: Cli) -> Result<Cli, ClapError> {
    match &cli.command {
        Commands::Start(start) => {
            if !start.modelspath.as_path().is_dir() {
                return Err(ClapError::raw(ValueValidation, format!("invalid path '{path}' for '--models-path <DIR>': '{path}' is not a directory", path=start.modelspath.display())).format(&mut Cli::command()));
            }
            if let Some(path_buf) = &start.storage_definitions {
                if !path_buf.is_file() {
                    return Err(ClapError::raw(ValueValidation, format!("invalid path '{path}' for '--storage-definitions <FILE>': '{path}' is not a file", path=path_buf.display())).format(&mut Cli::command()));
                }
            }
        },
        Commands::CreateModel(create) => {
            if !create.modelspath.as_path().is_dir() {
                return Err(ClapError::raw(ValueValidation, format!("invalid path '{path}' for '--models-path <DIR>': '{path}' is not a directory", path=create.modelspath.display())).format(&mut Cli::command()));
            }
        },
        Commands::ConfigureStorages(configure) => {
            let path_buf: &PathBuf = &configure.storage_definitions;
            if path_buf.file_name().is_none() || path_buf.parent().is_none() || !path_buf.parent().unwrap().is_dir() {
                return Err(ClapError::raw(ValueValidation, format!("invalid path '{path}' for '--storage-definitions <FILE>': '{path}' is not a file in an existing directory", path=path_buf.display())).format(&mut Cli::command()));
            }
        }
    }
    Ok(cli)
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse() {
        assert!(
            Cli::try_parse_from(vec!["gus", "start", "-p", "-1"]).is_err(),
            "Expected Error when passing negatives to -p"
        );
        assert!(
            Cli::try_parse_from(vec!["gus", "start", "-p", "65536"]).is_err(),
            "Expected Error when passing a port greater than maximum of 65535 to -p"
        );
        let mut args: Cli = Cli::try_parse_from(vec!["gus", "start"]).unwrap();
        assert!(
            validate_args(args).is_ok(),
            "Unexpected Error when parsing start args with default values"
        );
        args = Cli::try_parse_from(vec!["gus", "create-model"]).unwrap();
        assert!(
            validate_args(args).is_ok(),
            "Unexpected Error when parsing create-model args with default values"
        );
    }

    #[test]
    fn test_validate_args() {
        // start
        let mut args: Cli = Cli::try_parse_from(vec!["gus", "start", "-m", "./not_existing_dir/"]).unwrap();
        assert!(
            validate_args(args).is_err(),
            "Expected Error when passing a not existing directory to 'start -m'"
        );
        args = Cli::try_parse_from(vec!["gus", "start", "-m", "./Cargo.toml"]).unwrap();
        assert!(
            validate_args(args).is_err(),
            "Expected Error when passing a file to 'start -m'"
        );

        let mut args: Cli = Cli::try_parse_from(vec!["gus", "start", "-s", "./src/"]).unwrap();
        assert!(
            validate_args(args).is_err(),
            "Expected Error when passing an existing directory to 'start -s'"
        );
        args = Cli::try_parse_from(vec!["gus", "start", "-s", "./Cargo.toml.not.existing"]).unwrap();
        assert!(
            validate_args(args).is_err(),
            "Expected Error when passing a not existing file to 'start -s'"
        );


        // create-model
        args = Cli::try_parse_from(vec!["gus", "create-model", "-m", "./not_existing_dir/"]).unwrap();
        assert!(
            validate_args(args).is_err(),
            "Expected Error when passing a not existing directory to 'create-model -m'"
        );
        args = Cli::try_parse_from(vec!["gus", "create-model", "-m", "./Cargo.toml"]).unwrap();
        assert!(
            validate_args(args).is_err(),
            "Expected Error when passing a file to 'create-model -m'"
        );
    }
}