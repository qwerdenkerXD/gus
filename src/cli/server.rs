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
    Bytes as BodyBytes,
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
use model::{
    create_one,
    read_one
};

// used functions
use view::get_view_file;

pub async fn start(port: u16) -> Result<(), Error> {
    let server = HttpServer::new(|| 
        App::new().service(uri_handler_post)
                  .service(uri_handler_get)
                  .service(uri_handler_put)
                  .service(uri_handler_delete)
                  )
                  .bind(format!("127.0.0.1:{port}"))?;
    println!("Listening on port {port}");
    server.run().await
}

fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(JsonError{
        error: "This page does not exist".to_string()
    })
}

fn bad_request(message: String) -> HttpResponse {
    return HttpResponse::BadRequest().json(JsonError {
        error: message
    })
}

fn bad_endpoint() -> HttpResponse {
    bad_request("This endpoint does not exist".to_string())
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
    if segments.len() == 1 {
        match subroutes.as_str() {
            "" => return send_view_file(&"index.html".to_string()),
            "robots.txt" => return send_view_file(&"robots.txt".to_string()),
            _ => return not_found()
        }
    }

    match segments.remove(0) {
        "view" => send_view_file(subroutes),
        "api" => rest_api_get(subroutes),
        _ => not_found()
    }
}

fn send_view_file(subroutes: &String) -> HttpResponse {
    let view_file = get_view_file(subroutes);
    match view_file {
        Some((file, content_type)) => HttpResponse::Ok().insert_header(("content-type", content_type.as_str())).body(file),
        None => not_found()
    }
}

fn rest_api_get(uri: &String) -> HttpResponse {
    let segments: &mut Vec<&str> = &mut uri.split("/").collect();
    segments.remove(0);  // api
    if segments.len() != 2 {
        return bad_endpoint();
    }
    match read_one(&segments.remove(0).to_string(), &segments.remove(0).to_string()) {
        Ok(record) => HttpResponse::Ok().json(JsonData {
            data: record
        }),
        Err(err) => bad_request(format!("{}", err))
    }
}



#[post("/{uri:.*}")]
async fn uri_handler_post(body: BodyBytes, uri: UriParam<String>) -> HttpResponse {
    let subroutes: &String = &uri.into_inner();
    let segments: &mut Vec<&str> = &mut subroutes.split("/").collect();

    match segments.remove(0) {
        "api" => rest_api_post(subroutes, &body),
        _ => bad_endpoint()
    }
}

fn rest_api_post(uri: &String, body: &BodyBytes) -> HttpResponse {
    let body_str: Result<&str, std::str::Utf8Error> = std::str::from_utf8(body);
    if body_str.is_err() {
        return bad_request("Invalid input, accepting utf-8 only".to_string())
    }
    let segments: &mut Vec<&str> = &mut uri.split("/").collect();
    segments.remove(0);  // api
    if segments.len() != 1 {
        return bad_endpoint();
    }
    match create_one(&segments.remove(0).to_string(), &body_str.unwrap().to_string()) {
        Ok(record) => HttpResponse::Ok().json(JsonData {
            data: record
        }),
        Err(err) => bad_request(format!("{}", err))
    }
}



#[put("/{uri:.*}")]
async fn uri_handler_put(body: BodyBytes, uri: UriParam<String>) -> HttpResponse {
    bad_endpoint()
}



#[delete("/{uri:.*}")]
async fn uri_handler_delete(uri: UriParam<String>) -> HttpResponse {
    bad_endpoint()
}