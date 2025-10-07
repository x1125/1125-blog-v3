use std::fs;
use tide::{Request, Response};
use tide::prelude::*;
use crate::blog::config::Config;

#[derive(Debug, Deserialize)]
struct SaveData {
    file: String,
    content: String,
}

pub async fn ctrl_save(mut req: Request<Config>) -> tide::Result {
    let SaveData { file, content } = req.body_json().await?;

    let f = fs::write(format!("{}/{}", req.state().get_input_path().to_string_lossy(), file), content);
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