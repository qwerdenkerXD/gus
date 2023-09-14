mod cli;

fn main() {
    let args = cli::get_args();
    match args.command {
        cli::index::Commands::Start(cmd) => cli::start(cmd),
        cli::index::Commands::CreateModel(cmd) => cli::create_model(cmd)
    }
}