use crate::blog::config::{Config, DEFAULT_BRANCH};
use actix_web::{web, HttpResponse, Responder};
use git2::Repository;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Commit {
    message: String,
}

pub async fn ctrl_commit(
    runtime: web::Data<Config>,
    commit: web::Json<Commit>,
) -> actix_web::Result<impl Responder> {
    let message = commit.message.clone();
    
    let repo_path = runtime.get_input_path();
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "failed to open: {}",
                e
            )));
        }
    };

    let signature = match repo.signature() {
        Ok(signature) => signature,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "missing signature: {}",
                e
            )));
        }
    };

    let mut index = repo.index().unwrap();
    let tree = match index.write_tree() {
        Ok(tree) => tree,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "could not write index to tree: {}",
                e
            )));
        }
    };

    let branch = repo.revparse_single(DEFAULT_BRANCH).unwrap();
    let commit = branch.as_commit().unwrap();

    if let Err(e) = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message.as_str(),
        &repo.find_tree(tree).unwrap(),
        &[&commit],
    ) {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to push to remote: {}",
            e.message()
        )));
    }

    Ok(HttpResponse::NoContent().finish())
}
