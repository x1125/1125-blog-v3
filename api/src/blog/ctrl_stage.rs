use std::path::Path;
use git2::{IndexAddOption, Repository};
use tide::{Request, Response, StatusCode};
use crate::blog::config::{Config, DEFAULT_BRANCH};
use tide::prelude::*;
use crate::blog::error::http_error;

#[derive(Debug, Deserialize)]
struct StageFile {
    file: String,
    stage: bool,
}

pub async fn ctrl_stage(mut req: Request<Config>) -> tide::Result {
    let StageFile { file, stage } = req.body_json().await?;

    let path_str = format!("{}/{}", req.state().get_input_path().to_string_lossy(), file);
    let path = Path::new(path_str.as_str());
    if !path.exists() && file != "*" {
        return Ok(Response::builder(StatusCode::NotFound).build());
    }

    let repo_path = req.state().get_input_path();
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("failed to open: {}", e.message())));
        },
    };
    let mut index = match repo.index() {
        Ok(index) => index,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("failed to get index: {}", e.message())));
        },
    };

    if stage {
        match index.add_all([&file].iter(), IndexAddOption::DEFAULT, None) {
            Ok(()) => {},
            Err(e) => {
                return Ok(http_error(StatusCode::InternalServerError, format!("unable to add to index: {}", e.message())));
            },
        }
    } else {
        let reference = repo.find_reference(format!("refs/heads/{}", DEFAULT_BRANCH).as_str()).unwrap();
        let diff = repo
            .diff_tree_to_workdir_with_index(Some(&reference.peel_to_commit().unwrap().tree().unwrap()), None).unwrap();

        for diff_delta in diff.deltas().into_iter() {
            let file_path = diff_delta.old_file().path().unwrap();
            if file != "*" && file != file_path.to_string_lossy() {
                continue;
            }

            if let Err(e) = index.remove_path(file_path) {
                return Ok(http_error(StatusCode::InternalServerError, format!("could not remove from index: {}", e.message())));
            }
        }
    }

    match index.write() {
        Ok(()) => {},
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("could not remove from index: {}", e.message())));
        }
    }

    Ok(Response::builder(StatusCode::NoContent).build())
}