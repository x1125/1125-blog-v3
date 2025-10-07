use std::fs;
use tide::{Request, Response, StatusCode};
use tide::prelude::*;
use crate::blog::config::Config;
use crate::blog::error::http_error;

#[derive(Debug, Deserialize)]
struct SaveData {
    file: String,
    content: String,
}

pub async fn ctrl_save(mut req: Request<Config>) -> tide::Result {
    let SaveData { file, content } = req.body_json().await?;

    if let Err(e) = fs::write(format!("{}/{}", req.state().get_input_path().to_string_lossy(), file), content) {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to save: {}", e)));
    }

    Ok(Response::builder(StatusCode::NoContent).build())
}