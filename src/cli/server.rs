pub mod model;
mod view;


// used types
use std::str::Utf8Error;
use std::net::Ipv4Addr;
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

// used functions
use std::str::from_utf8;
use model::{
    create_one,
    read_one,
    update_one,
    delete_one
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

pub async fn start(port: u16, ip: Ipv4Addr) -> Result<(), Error> {
    let server = HttpServer::new(|| 
        App::new().service(uri_handler_post)
                  .service(uri_handler_get)
                  .service(uri_handler_put)
                  .service(uri_handler_delete)
                  )
                  .bind(format!("{ip}:{port}"))?;
    println!("Listening on {ip}:{port}");
    server.run().await
}

fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(JsonError{
        error: "This page does not exist".to_string()
    })
}

fn bad_request(message: String) -> HttpResponse {
    HttpResponse::BadRequest().json(JsonError {
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
    let subroutes: &str = &uri.into_inner();

    let segments: &mut Vec<&str> = &mut subroutes.split('/').collect();
    if segments.len() == 1 {
        match subroutes {
            "" => return send_view_file("index.html"),
            "robots.txt" => return send_view_file("robots.txt"),
            _ => return not_found()
        }
    }
    
    match segments.remove(0) {
        "view" => send_view_file(subroutes),
        "api" => {
            match segments.remove(0) {
                "rest" => rest_api_get(&segments.join("/")),
                "graphql" => send_view_file("graphql-gui.html"),
                _ => not_found()
            }
        },
        _ => not_found()
    }
}

fn send_view_file(subroutes: &str) -> HttpResponse {
    let view_file = get_view_file(subroutes);
    match view_file {
        Some((file, content_type)) => HttpResponse::Ok().insert_header(("content-type", content_type.as_str())).body(file),
        None => not_found()
    }
}

fn rest_api_get(uri: &str) -> HttpResponse {
    let segments: &mut Vec<&str> = &mut uri.split('/').collect();
    if segments.len() != 2 {
        return bad_endpoint();
    }
    let model_name: &str = segments.remove(0);
    let id: &str = segments.remove(0);
    match read_one(model_name, id) {
        Ok(record) => HttpResponse::Ok().json(JsonData {
            data: record
        }),
        Err(err) => bad_request(err.to_string())
    }
}



#[post("/{uri:.*}")]
async fn uri_handler_post(body: BodyBytes, uri: UriParam<String>) -> HttpResponse {
    let subroutes: &str = &uri.into_inner();
    let segments: &mut Vec<&str> = &mut subroutes.split('/').collect();

    if segments.len() == 1 {
        return bad_endpoint();
    }

    match segments.remove(0) {
        "api" => {
            match segments.remove(0) {
                "rest" => rest_api_post(&segments.join("/"), &body),
                "graphql" => bad_endpoint(),
                _ => bad_endpoint()
            }
        },
        _ => bad_endpoint()
    }
}

fn rest_api_post(uri: &str, body: &BodyBytes) -> HttpResponse {
    let body_str: Result<&str, Utf8Error> = from_utf8(body);
    if body_str.is_err() {
        return bad_request("Invalid body, accepting utf-8 only".to_string())
    }
    let segments: &mut Vec<&str> = &mut uri.split('/').collect();
    if segments.len() != 1 {
        return bad_endpoint();
    }
    match create_one(segments.remove(0), body_str.unwrap()) {
        Ok(record) => HttpResponse::Created().json(JsonData {
            data: record
        }),
        Err(err) => bad_request(err.to_string())
    }
}



#[put("/{uri:.*}")]
async fn uri_handler_put(body: BodyBytes, uri: UriParam<String>) -> HttpResponse {
    let subroutes: &str = &uri.into_inner();
    let segments: &mut Vec<&str> = &mut subroutes.split('/').collect();

    if segments.len() == 1 {
        return bad_endpoint();
    }

    match segments.remove(0) {
        "api" => {
            match segments.remove(0) {
                "rest" => rest_api_put(&segments.join("/"), &body),
                "graphql" => bad_endpoint(),
                _ => bad_endpoint()
            }
        },
        _ => bad_endpoint()
    }
}

fn rest_api_put(uri: &str, body: &BodyBytes) -> HttpResponse {
    let body_str: Result<&str, Utf8Error> = from_utf8(body);
    if body_str.is_err() {
        return bad_request("Invalid body, accepting utf-8 only".to_string())
    }
    let segments: &mut Vec<&str> = &mut uri.split('/').collect();
    if segments.len() != 2 {
        return bad_endpoint();
    }
    let model_name: &str = segments.remove(0);
    let id: &str = segments.remove(0);
    match update_one(model_name, id, body_str.unwrap()) {
        Ok(record) => HttpResponse::Ok().json(JsonData {
            data: record
        }),
        Err(err) => bad_request(err.to_string())
    }
}



#[delete("/{uri:.*}")]
async fn uri_handler_delete(uri: UriParam<String>) -> HttpResponse {
    let subroutes: &str = &uri.into_inner();
    let segments: &mut Vec<&str> = &mut subroutes.split('/').collect();

    if segments.len() == 1 {
        return bad_endpoint();
    }

    match segments.remove(0) {
        "api" => {
            match segments.remove(0) {
                "rest" => rest_api_delete(&segments.join("/")),
                "graphql" => bad_endpoint(),
                _ => bad_endpoint()
            }
        },
        _ => bad_endpoint()
    }
}

fn rest_api_delete(uri: &str) -> HttpResponse {
    let segments: &mut Vec<&str> = &mut uri.split('/').collect();
    if segments.len() != 2 {
        return bad_endpoint();
    }
    let model_name: &str = segments.remove(0);
    let id: &str = segments.remove(0);
    match delete_one(model_name, id) {
        Ok(record) => HttpResponse::Ok().json(JsonData {
            data: record
        }),
        Err(err) => bad_request(err.to_string())
    }
}




#[cfg(test)]
mod tests {
    use super::*;


    use actix_web::dev::ServiceResponse;
    use actix_web::test::TestRequest;
    use actix_web::body::MessageBody;
    use std::path::PathBuf;

    use std::fs::remove_file;
    use serde_json::from_str;
    use actix_web::test::{
        init_service,
        call_service
    };

    fn pre_test(file_name: &str) {
        if PathBuf::from(file_name).as_path().is_file() {
            assert!(remove_file(file_name).is_ok(), "Storage file {} already existing, unable to remove", file_name);
        }
    }

    fn post_test(file_name: &str) {
        if PathBuf::from(file_name).as_path().is_file() {
            assert!(remove_file(file_name).is_ok(), "Unable to remove storage file {} after test", file_name);
        }
    }

    #[actix_web::test]
    // not completed
    async fn test_rest_api_post() {
        const TEST_STORAGE_FILE: &str = "server.data.test.json";

        pre_test(TEST_STORAGE_FILE);

        let app = init_service(App::new().service(uri_handler_post)).await;

        let valid_input = r#"
            {
                "id": 1,
                "name": "Natural Born Killers",
                "year": 1994,
                "actors": ["Woody Harrelson", "Juliette Lewis"],
                "recommended": true
            }
        "#;
        let req = TestRequest::post().uri("/api/rest/movie")
                                     .set_payload(valid_input)
                                     .to_request();
        let res: ServiceResponse = call_service(&app, req).await;
        assert!(res.status().is_success(), "Unexpected error when creating a valid record");

        let expected: Record = from_str(valid_input).unwrap();
        let res_body: BodyBytes = res.into_body().try_into_bytes().unwrap();
        let res_data: JsonData = from_str(from_utf8(&res_body).unwrap()).unwrap();
        assert_eq!(res_data.data, expected, "Sent data doesn't match the response");

        post_test(TEST_STORAGE_FILE);
    }
}