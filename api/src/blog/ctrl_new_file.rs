use std::fs;
use std::path::Path;
use tide::{Request, Response};
use tide::prelude::*;
use crate::blog::config::Config;

#[derive(Debug, Deserialize)]
struct NewFile {
    file: String,
}

pub async fn ctrl_new_file(mut req: Request<Config>) -> tide::Result {
    let NewFile { file } = req.body_json().await?;

    let path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), file);
    let path = Path::new(path_str.as_str());
    if path.exists() {
        let response = Response::builder(409)
            .build();
        return Ok(response);
    }

    let f = fs::write(path, "");
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