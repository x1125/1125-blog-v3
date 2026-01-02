use crate::blog::config::Config;
use actix_web::{web, HttpResponse, Responder};
use base64::engine::general_purpose;
use base64::Engine;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct UploadData {
    name: String,
    size: i64,
    content: String,
}

pub async fn ctrl_upload(
    runtime: web::Data<Config>,
    upload_data: web::Json<UploadData>,
) -> actix_web::Result<impl Responder> {
    // body_form doesn't seem to work with file uploads...
    // TODO: check again with actix
    let name = upload_data.name.clone();
    let size = upload_data.size;
    let content = upload_data.content.clone();

    let decoded_content = match general_purpose::STANDARD.decode(content) {
        Ok(d) => d,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "unable to decode content: {}",
                e
            )))
        }
    };

    let path_str = format!("{}/{}", runtime.get_input_path().to_string_lossy(), name);
    let path = Path::new(path_str.as_str());
    if path.exists() {
        return Err(actix_web::error::ErrorConflict("file exists"));
    }

    match path.parent() {
        Some(p) => {
            if !p.exists() {
                if let Err(e) = fs::create_dir(p) {
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

    if size != decoded_content.len() as i64 {
        return Err(actix_web::error::ErrorUnprocessableEntity("size mismatch"));
    }

    if let Err(e) = fs::write(path, decoded_content) {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to write: {}",
            e
        )));
    }

    Ok(HttpResponse::Created().finish())
}
