mod index;
mod server;

pub use index::*;
use server::model::model_cli::create_model;
use server::model::configure_storages;

pub fn run() -> Option<impl std::future::Future<Output = Result<(), std::io::Error>>> {
    let cli: Result<Cli, ClapError> = get_validated_args();
    if let Err(err) = cli {
        if err.print().is_err() {
            eprintln!("{err}", err=err.render());
        }
        return None;
    }
    match cli.unwrap().command {
        Commands::Start(args) => return Some(server::start(args.port, args.bind)),
        Commands::CreateModel(args) => create_model(args),
        Commands::ConfigureStorages(args) => configure_storages(args)
    }

    None
}