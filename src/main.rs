mod cli;

#[actix_web::main]
async fn main() {
    match cli::run() {
        Some(server) => {
            if let Err(err) = server.await {
                eprintln!("{}", err);
                return;
            }
        },
        None => return,
    }
}