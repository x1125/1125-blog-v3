use crate::blog::utils::{get_changes, get_diffs, Change, Diff};
use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse, Responder};
use git2::Repository;
use serde::Serialize;
use serde_json::json;

use crate::Config;

#[derive(Serialize)]
pub struct ChangeResponse {
    pub changes: Vec<Change>,
    pub diffs: Vec<Diff>,
}

pub async fn ctrl_get_changes(runtime: web::Data<Config>) -> actix_web::Result<impl Responder> {
    let path = runtime.get_input_path();
    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "failed to open: {}",
                e.message()
            )));
        }
    };
    let change_response = json!(ChangeResponse {
        changes: get_changes(&repo),
        diffs: get_diffs(&repo),
    });

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(change_response.to_string()))
}
