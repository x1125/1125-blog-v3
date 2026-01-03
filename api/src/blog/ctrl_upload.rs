use crate::blog::config::Config;
use actix_multipart::form::json::Json;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::fs::{copy, create_dir, remove_file};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Metadata {
    name: String,
}

#[derive(MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
    json: Json<Metadata>,
}

pub async fn ctrl_upload(
    runtime: web::Data<Config>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> actix_web::Result<impl Responder> {
    let path_str = format!(
        "{}/{}",
        runtime.get_input_path().to_string_lossy(),
        form.json.name
    );
    let path = Path::new(path_str.as_str());
    if path.exists() {
        return Err(actix_web::error::ErrorConflict("file exists"));
    }

    match path.parent() {
        Some(p) => {
            if !p.exists() {
                if let Err(e) = create_dir(p) {
                    return Err(actix_web::error::ErrorInternalServerError(format!(
                        "unable to create dir: {}",
                        e
                    )));
                }
            }
        }
        None => {
            return Err(actix_web::error::ErrorInternalServerError(
                "Invalid directory",
            ));
        }
    }

    // file.persist() doesn't work, since it does a move
    // and fails due to cross fs (/tmp is tmpfs)
    // see: https://github.com/rwf2/Rocket/issues/1600
    if let Err(e) = copy(form.file.file.path(), &path) {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to copy uploaded file: {}",
            e
        )));
    }
    if let Err(e) = remove_file(form.file.file.path()) {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to remove temp file: {}",
            e
        )));
    }

    Ok(HttpResponse::Created().finish())
}
