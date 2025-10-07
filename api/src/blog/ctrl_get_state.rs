use std::collections::HashMap;
use serde_json::json;
use tide::{Request, Response};
use tide::http::mime;

use crate::lib::config::CONTENT_NAMES;
use crate::Config;
use crate::lib::utils::{Content, find_files, get_changes, get_entries};

pub async fn ctrl_get_state(req: Request<Config>) -> tide::Result {
    let mut contents = HashMap::new();
    for name in CONTENT_NAMES {
        let path = format!("{}/{}", req.state().working_path.to_string_lossy(), name);
        let mut files = find_files(path.clone(), None);
        let entries = get_entries(&mut files);
        let changes = get_changes(path);
        contents.insert(name, Content {
            entries,
            changes,
        });
    }

    let json_payload = json!(contents);

    let response = Response::builder(200)
        .body(json_payload)
        .content_type(mime::JSON)
        .build();
    Ok(response)
}
