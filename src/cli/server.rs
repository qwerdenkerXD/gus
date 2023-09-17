include!(concat!(env!("OUT_DIR"), "/view.rs"));

pub mod model;

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
    if subroutes.ends_with("/") {
        return not_found();
    }
    let segments: &mut Vec<&str> = &mut subroutes.split("/").collect();
    let mut view_files: ViewFiles = get_view_files();

    match segments.remove(0) {
        "" => send_view_file(&view_files, &URN::FileName("index.html".to_string())),
        "static" => {
            if let Hierarchy::Dir(dir) = view_files.get(&URN::DirName("static".to_string())).unwrap().clone() {
                view_files = dir.clone();
                while segments.len() > 0 {
                    if segments.len() > 1 {
                        if let Some(dir) = into_next_dir(&view_files, &URN::DirName(segments.remove(0).to_string())) {
                            view_files = dir;
                        } else {
                            return not_found();
                        }
                    } else {
                        return send_view_file(&view_files, &URN::FileName(segments.remove(0).to_string()));
                    }
                }
            }
            not_found()
        }
        other => send_view_file(&view_files, &URN::FileName(other.to_string()))
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
