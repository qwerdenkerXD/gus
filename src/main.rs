mod model;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[structopt(short, long, default_value = "./models", help = "The path to the model definitions")]
    modelspath: String,
}

fn main() {
    let args = Cli::parse();
    if let Err(_) = model::parse_models(args.modelspath.clone()) {
        println!("Warning: No models defined in {}", args.modelspath.clone());
    }
}