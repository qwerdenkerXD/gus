pub mod model;
mod view;


// used types
use model::Record;
use std::io::Error;
use actix_web::{
    HttpResponse,
    HttpServer,
    App
};
use actix_web::web::{
    Path as UriParam
};

// used derive macros
use serde_derive::{
    Deserialize,
    Serialize
};
use actix_web::{
    post,
    get,
    put,
    delete
};

// used functions
use view::get_view_file;

pub async fn start(port: u16) -> Result<(), Error> {
    let server = HttpServer::new(|| 
        App::new().service(uri_handler_get))
                  .bind(format!("127.0.0.1:{port}"))?;
    println!("Listening on port {port}");
    server.run().await
}

fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(JsonError{
        error: "This page does not exist".to_string()
    })
}

#[derive(Deserialize, Serialize, Debug)]
struct JsonError {
    error: String
}

#[derive(Deserialize, Serialize, Debug)]
struct JsonData {
    data: Record
}

#[get("/{uri:.*}")]
async fn uri_handler_get(uri: UriParam<String>) -> HttpResponse {
    let subroutes: &String = &uri.into_inner();
    let segments: &mut Vec<&str> = &mut subroutes.split("/").collect();

    match segments.remove(0) {
        "" => send_view_file(&"index.html".to_string()),
        "view" => send_view_file(subroutes),
        other => not_found()
    }
}

fn send_view_file(subroutes: &String) -> HttpResponse {
    let view_file = get_view_file(subroutes);
    match view_file {
        Some((file, content_type)) => HttpResponse::Ok().insert_header(("content-type", content_type.as_str())).body(file),
        None => not_found()
    }
}