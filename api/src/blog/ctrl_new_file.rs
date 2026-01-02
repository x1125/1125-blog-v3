use crate::blog::config::Config;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct NewFile {
    file: String,
}

pub async fn ctrl_new_file(
    runtime: web::Data<Config>,
    new_file: web::Json<NewFile>,
) -> actix_web::Result<impl Responder> {
    let path_str = format!(
        "{}/{}",
        runtime.get_input_path().to_string_lossy(),
        new_file.file.clone()
    );
    let path = Path::new(path_str.as_str());
    if path.exists() {
        return Err(actix_web::error::ErrorConflict("file already exists"));
    }

    if let Err(e) = fs::write(path, "") {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to write file: {}",
            e
        )));
    }

    Ok(HttpResponse::NoContent().finish())
}
