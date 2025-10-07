use crate::blog::config::{Config, DEFAULT_BRANCH, REF_NAME, REMOTE_NAME};
use crate::blog::error::http_error;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use serde::Serialize;
use serde_json::json;
use tide::http::mime;
use tide::{Request, Response, StatusCode};

#[derive(Debug, Serialize)]
pub struct PullResponse {
    pub message: String,
}

pub async fn ctrl_pull_remote(req: Request<Config>) -> tide::Result {
    let repo_path = req.state().get_input_path();
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("failed to open: {}", e.message())));
        }
    };

    let mut remote = match repo.find_remote(REMOTE_NAME) {
        Ok(remote) => remote,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("unable to find remote: {}", e.message())));
        }
    };

    let mut fetch_option = FetchOptions::new();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&req.state().git_ssh_key_path),
            None,
        )
    });
    fetch_option.remote_callbacks(callbacks);
    if let Err(e) = remote.fetch(&[REF_NAME], Some(&mut fetch_option), None) {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to pull from remote: {}", e.message())));
    }

    let fetch_head = match repo.find_reference("FETCH_HEAD") {
        Ok(fetch_head) => fetch_head,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("unable to find FETCH_HEAD: {}", e.message())));
        }
    };
    let fetch_commit = match repo.reference_to_annotated_commit(&fetch_head) {
        Ok(fetch_commit) => fetch_commit,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("unable to find fetch commit: {}", e.message())));
        }
    };

    let merge_analysis = match repo.merge_analysis(&[&fetch_commit]) {
        Ok(merge_analysis) => merge_analysis,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("unable to find merge analysis: {}", e.message())));
        }
    };

    if merge_analysis.0.is_up_to_date() {
        let json_payload = json!(PullResponse{
            message: "Already up to date".to_string(),
        });

        return Ok(Response::builder(StatusCode::Ok)
            .body(json_payload)
            .content_type(mime::JSON)
            .build());
    } else if merge_analysis.0.is_fast_forward() {
        println!("Fast-forwarding");
        let ref_name = format!("refs/heads/{}", DEFAULT_BRANCH);
        let mut reference = match repo.find_reference(&ref_name) {
            Ok(reference) => reference,
            Err(e) => {
                return Ok(http_error(StatusCode::InternalServerError, format!("unable to find reference: {}", e.message())));
            }
        };
        match reference.set_target(fetch_commit.id(), "Fast-Forward") {
            Ok(_) => {}
            Err(e) => {
                return Ok(http_error(StatusCode::InternalServerError, format!("unable to set target: {}", e.message())));
            }
        }
        match repo.set_head(&ref_name) {
            Ok(_) => {}
            Err(e) => {
                return Ok(http_error(StatusCode::InternalServerError, format!("unable to set head: {}", e.message())));
            }
        };
        match repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force())) {
            Ok(_) => {}
            Err(e) => {
                return Ok(http_error(StatusCode::InternalServerError, format!("unable to checkout head: {}", e.message())));
            }
        };
        let json_payload = json!(PullResponse{
            message: "Fast-forwarded".to_string(),
        });

        return Ok(Response::builder(StatusCode::Ok)
            .body(json_payload)
            .content_type(mime::JSON)
            .build());
    } else {
        return Ok(http_error(StatusCode::InternalServerError, "Merge needed"));
    }
}