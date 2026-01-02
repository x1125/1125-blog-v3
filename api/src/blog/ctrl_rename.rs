use crate::blog::config::Config;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct RenameFile {
    file: String,
    new_file: String,
}

pub async fn ctrl_rename(
    runtime: web::Data<Config>,
    rename_file: web::Json<RenameFile>,
) -> actix_web::Result<impl Responder> {
    let file = rename_file.file.clone();
    let new_file = rename_file.new_file.clone();

    let path_str = format!("{}/{}", runtime.get_input_path().to_string_lossy(), file);
    let path = Path::new(path_str.as_str());
    if !path.exists() {
        return Err(actix_web::error::ErrorNotFound("File not found"));
    }

    let new_path_str = format!(
        "{}/{}",
        runtime.get_input_path().to_string_lossy(),
        new_file
    );
    let new_path = Path::new(new_path_str.as_str());
    if new_path.exists() {
        return Err(actix_web::error::ErrorConflict("File already exists"));
    }

    if let Err(e) = fs::rename(path, new_path) {
        return Err(actix_web::error::ErrorInternalServerError(e));
    }

    Ok(HttpResponse::NoContent().finish())
}
