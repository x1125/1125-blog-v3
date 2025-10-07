use git2::Repository;
use tide::{Request, Response, StatusCode};
use tide::prelude::*;
use crate::blog::config::{Config, DEFAULT_BRANCH};
use crate::blog::error::http_error;

#[derive(Debug, Deserialize)]
struct Commit {
    message: String,
}

pub async fn ctrl_commit(mut req: Request<Config>) -> tide::Result {
    let Commit { message } = req.body_json().await?;

    let repo_path = req.state().get_input_path();
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("failed to open: {}", e)));
        },
    };

    let signature = match repo.signature() {
        Ok(signature) => signature,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("missing signature: {}", e)));
        }
    };

    let mut index = repo.index().unwrap();
    let tree = match index.write_tree() {
        Ok(tree) => tree,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError,format!("could not write index to tree: {}", e)));
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
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to push to remote: {}", e.message())));
    }

    Ok(Response::builder(StatusCode::NoContent).build())
}