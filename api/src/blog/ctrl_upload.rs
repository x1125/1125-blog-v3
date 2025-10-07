use crate::blog::config::Config;
use std::fs;
use std::path::Path;
use tide::prelude::*;
use tide::{Request, Response, StatusCode};
use crate::blog::error::http_error;

#[derive(Debug, Deserialize)]
struct UploadData {
    name: String,
    size: i64,
    content: String,
}

pub async fn ctrl_upload(mut req: Request<Config>) -> tide::Result {
    // body_form doesn't seem to work with file uploads...
    let UploadData {
        name,
        size,
        content,
    } = req.body_json().await?;

    let decoded_content = base64::decode(content)?;

    let path_str = format!(
        "{}/{}",
        req.state().get_input_path().to_string_lossy(),
        name
    );
    let path = Path::new(path_str.as_str());
    if path.exists() {
        return Ok(Response::builder(StatusCode::Conflict).build());
    }

    match path.parent() {
        Some(p) => {
            if !p.exists() {
                if let Err(e) = fs::create_dir(p) {
                    return Ok(http_error(StatusCode::InternalServerError, format!("unable to create dir: {}", e)));
                }
            }
        }
        None => {
            return Ok(http_error(StatusCode::InternalServerError, "Invalid directory"));
        }
    }

    if size != decoded_content.len() as i64 {
        return Ok(Response::builder(StatusCode::UnprocessableEntity).build());
    }

    if let Err(e) = fs::write(path, decoded_content) {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to write: {}", e)));
    }

    Ok(Response::builder(StatusCode::Created).build())
}