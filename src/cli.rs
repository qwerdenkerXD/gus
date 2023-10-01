mod index;
mod server;

pub use index::*;
use server::model::model_cli::{
    create_model
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