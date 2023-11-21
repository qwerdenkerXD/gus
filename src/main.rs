mod cli;

#[actix_web::main]
async fn main() {
    if let Some(server) = cli::run() {
        if let Err(err) = server.await {
            eprintln!("{err}");
        }
    }
}