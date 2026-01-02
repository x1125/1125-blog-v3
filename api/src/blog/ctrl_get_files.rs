use crate::blog::utils::{find_files, get_entries, Content};
use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use crate::Config;

pub async fn ctrl_get_files(runtime: web::Data<Config>) -> actix_web::Result<impl Responder> {
    let path = runtime.get_input_path();
    let mut files = find_files(&path, None);
    let (files, unknown_files) = get_entries(&mut files);
    let content = json!(Content {
        entries: files,
        unknown_entries: unknown_files,
    });

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(content.to_string()))
}
