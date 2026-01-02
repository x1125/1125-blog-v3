use crate::blog::config::{Config, REF_NAME, REMOTE_NAME};
use actix_web::{web, HttpResponse, Responder};
use git2::{Cred, PushOptions, RemoteCallbacks, Repository};

pub async fn ctrl_push_remote(runtime: web::Data<Config>) -> actix_web::Result<impl Responder> {
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

    let mut push_option = PushOptions::new();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&runtime.git_ssh_key_path),
            None,
        )
    });
    push_option.remote_callbacks(callbacks);
    if let Err(e) = remote.push(&[REF_NAME], Some(&mut push_option)) {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to push to remote: {}",
            e.message()
        )));
    }

    Ok(HttpResponse::NoContent().finish())
}
