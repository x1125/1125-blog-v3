use git2::Repository;
use tide::{Request, Response};
use tide::prelude::*;
use crate::blog::config::Config;

#[derive(Debug, Deserialize)]
struct Commit {
    message: String,
}

pub async fn ctrl_commit(mut req: Request<Config>) -> tide::Result {
    let Commit { message } = req.body_json().await?;

    let repo_path = req.state().get_input_path();
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    let signature = match repo.signature() {
        Ok(signature) => signature,
        Err(e) => {
            let response = Response::builder(500)
                .body(format!("Missing signature: {}", e))
                .build();
            return Ok(response);
        }
    };
    let mut index = repo.index().unwrap();
    let tree = match index.write_tree() {
        Ok(tree) => tree,
        Err(e) => {
            let response = Response::builder(500)
                .body(format!("Could not write index to tree: {}", e))
                .build();
            return Ok(response);
        }
    };
    let master = repo.revparse_single("master").unwrap();
    let commit = master.as_commit().unwrap();
    match repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message.as_str(),
        &repo.find_tree(tree).unwrap(),
        &[&commit],
    ) {
        Ok(_) => {}
        Err(e) => {
            let response = Response::builder(500)
                .body(format!("{}", e.message()))
                .build();
            return Ok(response);
        }
    }

    let response = Response::builder(204)
        .build();
    Ok(response)
}