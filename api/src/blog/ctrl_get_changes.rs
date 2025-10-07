use crate::blog::error::http_error;
use crate::blog::utils::{get_changes, get_diffs, Change, Diff};
use git2::Repository;
use serde::Serialize;
use serde_json::json;
use tide::http::mime;
use tide::{Request, Response, StatusCode};

use crate::Config;

#[derive(Debug, Serialize)]
pub struct ChangeResponse {
    pub changes: Vec<Change>,
    pub diffs: Vec<Diff>,
}

pub async fn ctrl_get_changes(req: Request<Config>) -> tide::Result {
    let path = req.state().get_input_path();
    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("failed to open: {}", e.message())));
        }
    };
    let change_response = ChangeResponse {
        changes: get_changes(&repo),
        diffs: get_diffs(&repo),
    };
    let json_payload = json!(change_response);

    Ok(Response::builder(StatusCode::Ok)
        .body(json_payload)
        .content_type(mime::JSON)
        .build())
}