use crate::blog::config::Config;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct DeleteFile {
    file: String,
}

pub async fn ctrl_delete(
    runtime: web::Data<Config>,
    delete_file: web::Json<DeleteFile>,
) -> actix_web::Result<impl Responder> {
    let file = delete_file.file.clone();

    let path_str = format!("{}/{}", runtime.get_input_path().to_string_lossy(), file);
    let path = Path::new(path_str.as_str());
    if !path.exists() {
        return Err(actix_web::error::ErrorNotFound("file not found"));
    }

    if let Err(e) = if path.is_dir() {
        fs::remove_dir(path)
    } else {
        fs::remove_file(path)
    } {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to remove: {}",
            e
        )));
    }

    Ok(HttpResponse::NoContent().finish())
}
