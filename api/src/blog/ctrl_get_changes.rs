use git2::Repository;
use crate::blog::utils::{Change, Diff, get_changes, get_diffs};
use serde_json::json;
use serde::Serialize;
use tide::http::mime;
use tide::{Request, Response};

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
        Err(e) => panic!("failed to open: {}", e),
    };
    let change_response = ChangeResponse{
        changes: get_changes(&repo),
        diffs: get_diffs(&repo),
    };
    let json_payload = json!(change_response);

    let response = Response::builder(200)
        .body(json_payload)
        .content_type(mime::JSON)
        .build();
    Ok(response)
}
