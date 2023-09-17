include!(concat!(env!("OUT_DIR"), "/view.rs"));

pub mod model;

use actix_web::{
    post,
    get,
    put,
    delete
};
use actix_web::{
    App,
    HttpResponse,
    HttpServer
};
use actix_web::web::{
    Json,
    Path as UriParam
};
use serde_derive::{
    Deserialize,
    Serialize
};
use model::Record;
use std::io::Error;
use std::str::Split;

pub async fn start(port: u16) -> Result<(), Error> {
    let server = HttpServer::new(|| 
        App::new().service(get_uri_handler))
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
async fn get_uri_handler(uri: UriParam<String>) -> HttpResponse {
    let subroutes: &String = &uri.into_inner();
    let segments: &mut Vec<&str> = &mut subroutes.split("/").collect();
    let mut view_files: ViewFiles = get_view_files();

    match segments.remove(0) {
        "" => send_view_file(&view_files, &"index.html".to_string()),
        "static" => {
            if let Hierarchy::Dir(dir) = view_files.get("static").unwrap().clone() {
                view_files = dir.clone();
                while segments.len() > 0 {
                    if segments.len() > 1 {
                        if let Some(dir) = into_next_dir(&view_files, &segments.remove(0).to_string()) {
                            view_files = dir;
                        } else {
                            return not_found();
                        }
                    } else {
                        return send_view_file(&view_files, &segments.remove(0).to_string());
                    }
                }
            }
            not_found()
        }
        other => send_view_file(&view_files, &other.to_string())
    }
}

fn send_view_file(view_files: &ViewFiles, urn: &URN) -> HttpResponse {
    if let Some(entry) = view_files.get(urn) {
        if let Hierarchy::File((file, content_type)) = entry {
            return HttpResponse::Ok().insert_header(("content-type", content_type.as_str())).body(*file);
        }
    }
    not_found()
}

fn into_next_dir(view_files: &ViewFiles, urn: &URN) -> Option<ViewFiles> {
    if let Some(entry) = view_files.get(urn) {
        if let Hierarchy::Dir(dir) = entry {
            return Some(dir.clone());
        }
    }
    None
}
