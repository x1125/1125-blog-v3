use crate::blog::config::Config;
use crate::blog::error::http_error;
use std::fs;
use std::path::Path;
use tide::prelude::*;
use tide::{Request, Response, StatusCode};

#[derive(Debug, Deserialize)]
struct RenameFile {
    file: String,
    new_file: String,
}

pub async fn ctrl_rename(mut req: Request<Config>) -> tide::Result {
    let RenameFile { file, new_file } = req.body_json().await?;

    let path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), file);
    let path = Path::new(path_str.as_str());
    if !path.exists() {
        return Ok(Response::builder(StatusCode::NotFound).build());
    }

    let new_path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), new_file);
    let new_path = Path::new(new_path_str.as_str());
    if new_path.exists() {
        return Ok(Response::builder(StatusCode::Conflict).build());
    }

    if let Err(e) = fs::rename(path, new_path) {
        return Ok(http_error(StatusCode::InternalServerError, format!("{}", e)));
    }

    Ok(Response::builder(StatusCode::NoContent).build())
}