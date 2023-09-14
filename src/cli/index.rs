use clap::{ Parser, Subcommand };

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
    #[structopt(short, long, default_value = "./models", help = "The path to the model definitions")]
    pub modelspath: String,
}

#[derive(Parser, Debug)]
#[clap(name = "create-model", about = "An interactive Dialog to create valid model definitions")]
pub struct CreateModel {
    #[structopt(short, long, default_value = "./models", help = "The path to the model definitions")]
    pub modelspath: String,
}