use crate::blog::config::{Config, DEFAULT_BRANCH};
use crate::blog::utils::{get_changes, Change};
use actix_web::{web, HttpResponse, Responder};
use git2::build::CheckoutBuilder;
use git2::Repository;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct RevertFile {
    file: String,
}

pub async fn ctrl_revert(
    runtime: web::Data<Config>,
    revert_file: web::Json<RevertFile>,
) -> actix_web::Result<impl Responder> {
    let file = revert_file.file.clone();

    let repo_path = runtime.get_input_path();
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "failed to open: {}",
                e.message()
            )))
        }
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
        return Err(actix_web::error::ErrorNotFound("not found"));
    }

    let change = change.unwrap();
    if change.change == "Added" {
        let path_str = format!("{}/{}", runtime.get_input_path().to_string_lossy(), file);
        let path = Path::new(path_str.as_str());
        if !path.exists() {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "{} was not found",
                path_str
            )));
        }
        if let Err(e) = index.remove_path(Path::new(&file)) {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "could not unstage: {}",
                e
            )));
        }
    } else if change.change == "Renamed" {
        let path_str = format!("{}/{}", runtime.get_input_path().to_string_lossy(), file);
        let path = Path::new(path_str.as_str());
        if !path.exists() {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "{} was not found",
                path_str
            )));
        }

        let old_name = change.old_name.as_ref().unwrap();
        let old_path_str = format!(
            "{}/{}",
            runtime.get_input_path().to_string_lossy(),
            old_name
        );
        let old_path = Path::new(old_path_str.as_str());
        if old_path.exists() {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "{} already exists",
                old_path_str
            )));
        }

        if let Err(e) = fs::rename(path, old_path) {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "could not rename: {}",
                e
            )));
        }
    } else if change.change == "Deleted" {
        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.path(Path::new(file.as_str()));
        checkout_builder.recreate_missing(true);

        if let Err(e) = repo.checkout_head(Some(&mut checkout_builder)) {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "Unable to checkout file: {}",
                e
            )));
        }
    } else if change.change == "Modified" {
        let branch = repo.revparse_single(DEFAULT_BRANCH).unwrap();
        let commit = branch.as_commit().unwrap();
        let tree = commit.tree().unwrap();
        match tree.get_name(file.as_str()) {
            Some(tree_entry) => {
                let path_str = format!("{}/{}", runtime.get_input_path().to_string_lossy(), file);
                let path = Path::new(path_str.as_str());
                if !path.exists() {
                    return Err(actix_web::error::ErrorConflict("conflict"));
                }

                if let Err(e) = fs::write(
                    path,
                    tree_entry
                        .to_object(&repo)
                        .unwrap()
                        .as_blob()
                        .unwrap()
                        .content(),
                ) {
                    return Err(actix_web::error::ErrorInternalServerError(format!(
                        "unable to write: {}",
                        e
                    )));
                }
            }
            None => {
                return Err(actix_web::error::ErrorNotFound("file not found"));
            }
        };
    } else if change.change == "Untracked" {
        let path_str = format!("{}/{}", runtime.get_input_path().to_string_lossy(), file);
        let path = Path::new(path_str.as_str());
        if !path.exists() {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "{} was not found",
                path_str
            )));
        }

        // rename is not recognized, since not on the index

        // add happened
        if path.is_dir() {
            if let Err(e) = fs::remove_dir(path) {
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "could not delete: {}",
                    e
                )));
            }
        } else {
            if let Err(e) = fs::remove_file(path) {
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "could not delete: {}",
                    e
                )));
            }
        }
    } else {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "{} not implemented",
            change.change
        )));
    }

    // update index
    if let Err(e) = index.update_all(["*"].iter(), None) {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to update index: {}",
            e
        )));
    }
    if let Err(e) = index.write() {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to write index: {}",
            e
        )));
    }

    Ok(HttpResponse::NoContent().finish())
}
