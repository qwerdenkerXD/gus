// used types
use std::path::PathBuf;
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
    #[clap(name = "storage-definitions", short, long, value_name = "FILE", value_hint = FilePath, help = "The path to the storage definitions' file")]
    pub storage_definitions: Option<PathBuf>
}

#[derive(Parser, Debug)]
#[clap(name = "create-model", about = "An interactive Dialog to create valid model definitions")]
pub struct CreateModel {
    #[clap(name = "models-path", short, long, default_value = "./", value_name = "DIR", value_hint = DirPath, help = "The path to the model definitions")]
    pub modelspath: PathBuf
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

fn validate_args(mut cli: Cli) -> Result<Cli, ClapError> {
    match cli.command {
        Commands::Start(ref mut start) => {
            if !start.modelspath.as_path().is_dir() {
                return Err(ClapError::raw(ValueValidation, format!("invalid path '{}' for '--models-path <DIR>': '{}' is not a directory", &start.modelspath.display(), &start.modelspath.display())).format(&mut Cli::command()));
            }
            if let Some(path_buf) = &start.storage_definitions {
                if !path_buf.as_path().is_file() {
                    return Err(ClapError::raw(ValueValidation, format!("invalid path '{}' for '--storage-definitions <FILE>': '{}' is not a file", &path_buf.display(), &path_buf.display())).format(&mut Cli::command()));
                }
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