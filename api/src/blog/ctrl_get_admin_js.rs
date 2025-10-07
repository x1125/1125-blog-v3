use std::fs;
use tide::{Request, Response};
use tide::http::mime;
use crate::Config;

pub async fn ctrl_get_admin_js(req: Request<Config>) -> tide::Result {
    let admin_js_path = format!("{}/../admin.js", req.state().working_path.to_string_lossy());

    let response = Response::builder(200)
        .body(fs::read_to_string(admin_js_path)?)
        .content_type(mime::JAVASCRIPT)
        .build();
    Ok(response)
}