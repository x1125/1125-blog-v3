use crate::blog::config::Config;
use crate::blog::error::http_error;
use std::fs;
use std::path::Path;
use tide::prelude::*;
use tide::{Request, Response, StatusCode};

#[derive(Debug, Deserialize)]
struct NewFile {
    file: String,
}

pub async fn ctrl_new_file(mut req: Request<Config>) -> tide::Result {
    let NewFile { file } = req.body_json().await?;

    let path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), file);
    let path = Path::new(path_str.as_str());
    if path.exists() {
        return Ok(Response::builder(StatusCode::Conflict).build());
    }

    if let Err(e) = fs::write(path, "") {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to write file: {}", e)));
    }

    Ok(Response::builder(StatusCode::NoContent).build())
}