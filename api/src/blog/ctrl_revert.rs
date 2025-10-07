use std::fs;
use std::path::Path;
use git2::Repository;
use git2::build::CheckoutBuilder;
use tide::{Body, Request, Response, StatusCode};
use crate::blog::config::Config;
use crate::blog::utils::{Change, get_changes};
use tide::prelude::*;

#[derive(Debug, Deserialize)]
struct RevertFile {
    file: String,
}

pub async fn ctrl_revert(mut req: Request<Config>) -> tide::Result {
    let RevertFile { file } = req.body_json().await?;

    let repo_path = req.state().get_input_path();
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    let mut index = repo.index().unwrap();

    let changes = get_changes(&repo);
    let mut change: Option<&Change> = None;
    for ch in changes.iter() {
        if ch.name == file {
            change = Some(ch);
        }
    }

    if change.is_none() {
        let response = Response::builder(StatusCode::NotFound)
            .build();
        return Ok(response);
    }

    let change = change.unwrap();
    if change.change == "Added" {
        let path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), file);
        let path = Path::new(path_str.as_str());
        if !path.exists() {
            return Ok(http_error(StatusCode::InternalServerError, format!("{} was not found", path_str)));
        }
        if let Err(err) = index.remove_path(Path::new(&file)) {
            return Ok(http_error(StatusCode::InternalServerError, format!("could not unstage: {}", err)));
        }
    } else if change.change == "Renamed" {
        let path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), file);
        let path = Path::new(path_str.as_str());
        if !path.exists() {
            return Ok(http_error(StatusCode::InternalServerError, format!("{} was not found", path_str)));
        }

        let old_name = change.old_name.as_ref().unwrap();
        let old_path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), old_name);
        let old_path = Path::new(old_path_str.as_str());
        if old_path.exists() {
            return Ok(http_error(StatusCode::InternalServerError, format!("{} already exists", old_path_str)));
        }

        if let Err(err) = fs::rename(path, old_path) {
            return Ok(http_error(StatusCode::InternalServerError, format!("could not rename: {}", err)));
        }
    } else if change.change == "Deleted" {
        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.path(Path::new(file.as_str()));
        checkout_builder.recreate_missing(true);

        if let Err(err) = repo.checkout_head(Some(&mut checkout_builder)) {
            return Ok(http_error(StatusCode::InternalServerError, format!("Unable to checkout file: {}", err)));
        }
    } else if change.change == "Modified" {
        let master = repo.revparse_single("master").unwrap();
        let commit = master.as_commit().unwrap();
        let tree = commit.tree().unwrap();
        match tree.get_name(file.as_str()) {
            Some(tree_entry) => {
                let path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), file);
                let path = Path::new(path_str.as_str());
                if !path.exists() {
                    let response = Response::builder(409)
                        .build();
                    return Ok(response);
                }

                let f = fs::write(path, tree_entry.to_object(&repo).unwrap().as_blob().unwrap().content());
                if f.is_err() {
                    let response = Response::builder(500)
                        .body(format!("{}", f.err().unwrap()))
                        .build();
                    return Ok(response);
                }
            }
            None => {
                let response = Response::builder(StatusCode::NotFound)
                    .build();
                return Ok(response);
            }
        };
    } else if change.change == "Untracked" {
        let path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), file);
        let path = Path::new(path_str.as_str());
        if !path.exists() {
            return Ok(http_error(StatusCode::InternalServerError, format!("{} was not found", path_str)));
        }

        // rename is not recognized, since not on the index

        // add happened
        if path.is_dir() {
            if let Err(err) = fs::remove_dir(path) {
                return Ok(http_error(StatusCode::InternalServerError, format!("could not delete: {}", err)));
            }
        } else {
            if let Err(err) = fs::remove_file(path) {
                return Ok(http_error(StatusCode::InternalServerError, format!("could not delete: {}", err)));
            }
        }
    } else {
        return Ok(http_error(StatusCode::InternalServerError, format!("{} not implemented", change.change)));
    }

    // update index
    if let Err(err) = index.update_all(["*"].iter(), None) {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to update index: {}", err)));
    }
    if let Err(err) = index.write() {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to write index: {}", err)));
    }

    let response = Response::builder(204)
        .build();
    return Ok(response);
}

fn http_error(status: StatusCode, body: impl Into<Body>) -> Response {
    let response = Response::builder(status)
        .body(body)
        .build();
    return response;
}