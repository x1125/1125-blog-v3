use crate::blog::config::Config;
use std::fs;
use std::path::Path;
use tide::prelude::*;
use tide::{Request, Response};

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
        let response = Response::builder(409).build();
        return Ok(response);
    }

    match path.parent() {
        Some(p) => {
            if !p.exists() {
                let f = fs::create_dir(p);
                if f.is_err() {
                    let response = Response::builder(500)
                        .body(format!("{}", f.err().unwrap()))
                        .build();
                    return Ok(response);
                }
            }
        }
        None => {
            let response = Response::builder(500).body("Invalid directory").build();
            return Ok(response);
        }
    }

    if size != decoded_content.len() as i64 {
        let response = Response::builder(422).build();
        return Ok(response);
    }

    let f = fs::write(path, decoded_content);
    if f.is_err() {
        let response = Response::builder(500)
            .body(format!("{}", f.err().unwrap()))
            .build();
        return Ok(response);
    }

    let response = Response::builder(201).build();
    Ok(response)
}
