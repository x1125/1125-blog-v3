use std::fs;
use std::path::Path;
use tide::{Request, Response, StatusCode};
use tide::prelude::*;
use crate::blog::config::Config;
use crate::blog::error::http_error;

#[derive(Debug, Deserialize)]
struct DeleteFile {
    file: String,
}

pub async fn ctrl_delete(mut req: Request<Config>) -> tide::Result {
    let DeleteFile { file } = req.body_json().await?;

    let path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), file);
    let path = Path::new(path_str.as_str());
    if !path.exists() {
        return Ok(Response::builder(StatusCode::NotFound).build());
    }

    if let Err(e) = if path.is_dir() {
        fs::remove_dir(path)
    } else {
        fs::remove_file(path)
    } {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to remove: {}", e)));
    }

    Ok(Response::builder(StatusCode::NoContent).build())
}