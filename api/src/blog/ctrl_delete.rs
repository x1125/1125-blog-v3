use std::fs;
use std::path::Path;
use tide::{Request, Response};
use tide::prelude::*;
use crate::blog::config::Config;

#[derive(Debug, Deserialize)]
struct DeleteFile {
    file: String,
}

pub async fn ctrl_delete(mut req: Request<Config>) -> tide::Result {
    let DeleteFile { file } = req.body_json().await?;

    let path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), file);
    let path = Path::new(path_str.as_str());
    if !path.exists() {
        let response = Response::builder(404)
            .build();
        return Ok(response);
    }

    let f = if path.is_dir() {
        fs::remove_dir(path)
    } else {
        fs::remove_file(path)
    };
    if f.is_err() {
        let response = Response::builder(500)
            .body(format!("{}", f.err().unwrap()))
            .build();
        return Ok(response);
    }

    let response = Response::builder(204)
        .build();
    Ok(response)
}