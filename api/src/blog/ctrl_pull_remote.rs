use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use tide::{Request, Response, StatusCode};
use crate::blog::config::{Config, REMOTE_REF, REMOTE_NAME};
use crate::blog::error::http_error;

pub async fn ctrl_pull_remote(req: Request<Config>) -> tide::Result {
    let repo_path = req.state().get_input_path();
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("failed to open: {}", e.message())));
        },
    };

    let mut remote = match repo.find_remote(REMOTE_NAME) {
        Ok(remote) => remote,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("unable to find remote: {}", e.message())));
        },
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
    if let Err(e) = remote.fetch(&[REMOTE_REF], Some(&mut fetch_option), None) {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to pull from remote: {}", e.message())));
    }

    Ok(Response::builder(StatusCode::NoContent).build())
}