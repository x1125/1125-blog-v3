use git2::Repository;
use tide::{Request, Response, StatusCode};
use crate::blog::config::{Config, DEFAULT_BRANCH};
use crate::blog::error::http_error;

pub async fn ctrl_push_remote(req: Request<Config>) -> tide::Result {
    let repo_path = req.state().get_input_path();
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("failed to open: {}", e.message())));
        },
    };

    let mut remote = match repo.find_remote("origin") {
        Ok(remote) => remote,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("unable to find remote: {}", e.message())));
        },
    };

    if let Err(e) = remote.push(&[DEFAULT_BRANCH], None) {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to push to remote: {}", e.message())));
    }

    Ok(Response::builder(StatusCode::NoContent).build())
}