use std::path::Path;
use git2::{IndexAddOption, Repository};
use tide::{Request, Response};
use crate::blog::config::Config;
use tide::prelude::*;

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
        let response = Response::builder(404)
            .build();
        return Ok(response);
    }

    let repo_path = req.state().get_input_path();
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    let mut index = match repo.index() {
        Ok(index) => index,
        Err(e) => panic!("failed to get index: {}", e),
    };

    if stage {
        match index.add_all([&file].iter(), IndexAddOption::DEFAULT, None) {
            Ok(()) => {},
            Err(e) => {
                let response = Response::builder(500)
                    .body(format!("{}", e))
                    .build();
                return Ok(response);
            }
        }
    } else {
        let reference = repo.find_reference("refs/heads/master").unwrap();
        let diff = repo
            .diff_tree_to_workdir_with_index(Some(&reference.peel_to_commit().unwrap().tree().unwrap()), None).unwrap();

        for diff_delta in diff.deltas().into_iter() {
            let file_path = diff_delta.old_file().path().unwrap();
            if file != "*" && file != file_path.to_string_lossy() {
                continue;
            }

            if let Err(err) = index.remove_path(file_path) {
                let response = Response::builder(500)
                    .body(format!("could not remove from index: {}", err))
                    .build();
                return Ok(response);
            }
        }
    }

    match index.write() {
        Ok(()) => {},
        Err(e) => {
            let response = Response::builder(500)
                .body(format!("{}", e))
                .build();
            return Ok(response);
        }
    }

    let response = Response::builder(204)
        .build();
    Ok(response)
}