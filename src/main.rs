mod cli;
mod model;

use clap::Parser;

fn main() {
    let args = cli::Cli::parse();
    match args.command {
        cli::Commands::Start(cmd) => start(cmd),
        cli::Commands::CreateModel(cmd) => create_model(cmd)
    }
}

fn start(args: cli::StartServer) {
    if let Err(_) = model::parse_models(args.modelspath.clone()) {
        println!("Warning: No models defined in {}", args.modelspath.clone());
    }
}

fn create_model(args: cli::CreateModel) {
    unimplemented!()
}