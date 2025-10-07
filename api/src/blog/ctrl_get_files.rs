use crate::blog::utils::{find_files, get_entries, Content};
use serde_json::json;
use tide::http::mime;
use tide::{Request, Response, StatusCode};

use crate::Config;

pub async fn ctrl_get_files(req: Request<Config>) -> tide::Result {
    let path = req.state().get_input_path();
    let mut files = find_files(&path, None);
    let (files, unknown_files) = get_entries(&mut files);
    let content = Content {
        entries: files,
        unknown_entries: unknown_files,
    };

    Ok(Response::builder(StatusCode::Ok)
        .body(json!(content))
        .content_type(mime::JSON)
        .build())
}