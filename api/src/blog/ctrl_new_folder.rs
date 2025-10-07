use std::fs;
use std::path::Path;
use tide::{Request, Response, StatusCode};
use tide::prelude::*;
use crate::blog::config::Config;
use crate::blog::error::http_error;

#[derive(Debug, Deserialize)]
struct NewFolder {
    folder: String,
}

pub async fn ctrl_new_folder(mut req: Request<Config>) -> tide::Result {
    let NewFolder { folder } = req.body_json().await?;

    let path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), folder);
    let path = Path::new(path_str.as_str());
    if path.exists() {
        return Ok(Response::builder(StatusCode::Conflict).build());
    }

    if let Err(e) = fs::create_dir(path) {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to create dir: {}", e)));
    }

    Ok(Response::builder(StatusCode::NoContent).build())
}