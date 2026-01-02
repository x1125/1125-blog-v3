use crate::blog::config::Config;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct SaveData {
    file: String,
    content: String,
}

pub async fn ctrl_save(
    runtime: web::Data<Config>,
    save_data: web::Json<SaveData>,
) -> actix_web::Result<impl Responder> {
    let file = save_data.file.clone();
    let content = save_data.content.clone();

    if let Err(e) = fs::write(
        format!("{}/{}", runtime.get_input_path().to_string_lossy(), file),
        content,
    ) {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to save: {}",
            e
        )));
    }

    Ok(HttpResponse::NoContent().finish())
}
