use crate::blog::config::{Config, DEFAULT_BRANCH, REF_NAME, REMOTE_NAME};
use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse, Responder};
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
pub struct PullResponse {
    pub message: String,
}

pub async fn ctrl_pull_remote(runtime: web::Data<Config>) -> actix_web::Result<impl Responder> {
    let repo_path = runtime.get_input_path();
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "failed to open: {}",
                e.message()
            )));
        }
    };

    let mut remote = match repo.find_remote(REMOTE_NAME) {
        Ok(remote) => remote,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "unable to find remote: {}",
                e.message()
            )));
        }
    };

    let mut fetch_option = FetchOptions::new();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&runtime.git_ssh_key_path),
            None,
        )
    });
    fetch_option.remote_callbacks(callbacks);
    if let Err(e) = remote.fetch(&[REF_NAME], Some(&mut fetch_option), None) {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to pull from remote: {}",
            e.message()
        )));
    }

    let fetch_head = match repo.find_reference("FETCH_HEAD") {
        Ok(fetch_head) => fetch_head,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "unable to find FETCH_HEAD: {}",
                e.message()
            )));
        }
    };
    let fetch_commit = match repo.reference_to_annotated_commit(&fetch_head) {
        Ok(fetch_commit) => fetch_commit,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "unable to find fetch commit: {}",
                e.message()
            )));
        }
    };

    let merge_analysis = match repo.merge_analysis(&[&fetch_commit]) {
        Ok(merge_analysis) => merge_analysis,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "unable to find merge analysis: {}",
                e.message()
            )));
        }
    };

    if merge_analysis.0.is_up_to_date() {
        let json_payload = json!(PullResponse {
            message: "Already up to date".to_string(),
        });

        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(json_payload.to_string()))
    } else if merge_analysis.0.is_fast_forward() {
        println!("Fast-forwarding");
        let ref_name = format!("refs/heads/{}", DEFAULT_BRANCH);
        let mut reference = match repo.find_reference(&ref_name) {
            Ok(reference) => reference,
            Err(e) => {
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "unable to find reference: {}",
                    e.message()
                )));
            }
        };
        match reference.set_target(fetch_commit.id(), "Fast-Forward") {
            Ok(_) => {}
            Err(e) => {
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "unable to set target: {}",
                    e.message()
                )));
            }
        }
        match repo.set_head(&ref_name) {
            Ok(_) => {}
            Err(e) => {
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "unable to set head: {}",
                    e.message()
                )));
            }
        };
        match repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force())) {
            Ok(_) => {}
            Err(e) => {
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "unable to checkout head: {}",
                    e.message()
                )));
            }
        };
        let json_payload = json!(PullResponse {
            message: "Fast-forwarded".to_string(),
        });

        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(json_payload.to_string()))
    } else {
        Err(actix_web::error::ErrorForbidden("Merge needed"))
    }
}
