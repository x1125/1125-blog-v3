use crate::blog::config::Config;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct NewFolder {
    folder: String,
}

pub async fn ctrl_new_folder(
    runtime: web::Data<Config>,
    new_folder: web::Json<NewFolder>,
) -> actix_web::Result<impl Responder> {
    let folder = new_folder.folder.clone();

    let path_str = format!("{}/{}", runtime.get_input_path().to_string_lossy(), folder);
    let path = Path::new(path_str.as_str());
    if path.exists() {
        return Ok(HttpResponse::Conflict().finish());
    }

    if let Err(e) = fs::create_dir(path) {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to create dir: {}",
            e
        )));
    }

    Ok(HttpResponse::NoContent().finish())
}
