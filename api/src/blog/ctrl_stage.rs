use crate::blog::config::{Config, DEFAULT_BRANCH};
use actix_web::{web, HttpResponse, Responder};
use git2::{IndexAddOption, Repository};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct StageFile {
    file: String,
    stage: bool,
}

pub async fn ctrl_stage(
    runtime: web::Data<Config>,
    stage_file: web::Json<StageFile>,
) -> actix_web::Result<impl Responder> {
    let file = stage_file.file.clone();
    let stage = stage_file.stage;

    let path_str = format!("{}/{}", runtime.get_input_path().to_string_lossy(), file);
    let path = Path::new(path_str.as_str());
    if !path.exists() && file != "*" {
        return Err(actix_web::error::ErrorNotFound("file not found"));
    }

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
    let mut index = match repo.index() {
        Ok(index) => index,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "failed to get index: {}",
                e.message()
            )));
        }
    };

    if stage {
        match index.add_all([&file].iter(), IndexAddOption::DEFAULT, None) {
            Ok(()) => {}
            Err(e) => {
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "unable to add to index: {}",
                    e.message()
                )));
            }
        }
    } else {
        let reference = repo
            .find_reference(format!("refs/heads/{}", DEFAULT_BRANCH).as_str())
            .unwrap();
        let diff = repo
            .diff_tree_to_workdir_with_index(
                Some(&reference.peel_to_commit().unwrap().tree().unwrap()),
                None,
            )
            .unwrap();

        for diff_delta in diff.deltas().into_iter() {
            let file_path = diff_delta.old_file().path().unwrap();
            if file != "*" && file != file_path.to_string_lossy() {
                continue;
            }

            if let Err(e) = index.remove_path(file_path) {
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "could not remove from index: {}",
                    e.message()
                )));
            }
        }
    }

    match index.write() {
        Ok(()) => {}
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "could not remove from index: {}",
                e.message()
            )));
        }
    }

    Ok(HttpResponse::NoContent().finish())
}
