mod cli;

#[actix_web::main]
async fn main() {
    match cli::run() {
        Some(server) => {
            if let Err(_) = server.await {
                return;
            }
        },
        None => return,
    }
}