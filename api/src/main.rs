mod blog;

use std::env;
use clap::Command;
use std::path::Path;
use std::process;
use tera::Tera;

use tide_rustls::TlsListener;
use crate::blog::auth_middleware::AuthMiddleware;

use crate::blog::config::Config;
use crate::blog::ctrl_commit::ctrl_commit;
use crate::blog::ctrl_generate::ctrl_generate;
use crate::blog::ctrl_delete::ctrl_delete;
use crate::blog::ctrl_get_changes::ctrl_get_changes;
use crate::blog::ctrl_get_files::ctrl_get_files;
use crate::blog::ctrl_get_preview::ctrl_get_preview;
use crate::blog::ctrl_new_file::ctrl_new_file;
use crate::blog::ctrl_new_folder::ctrl_new_folder;
use crate::blog::ctrl_pull_remote::ctrl_pull_remote;
use crate::blog::ctrl_push_remote::ctrl_push_remote;
use crate::blog::ctrl_rename::ctrl_rename;
use crate::blog::ctrl_revert::ctrl_revert;
use crate::blog::ctrl_save::ctrl_save;
use crate::blog::ctrl_stage::ctrl_stage;
use crate::blog::ctrl_upload::ctrl_upload;
use crate::blog::generator::generate_all;

#[async_std::main]
async fn main() {
    let config = match Config::new() {
        Ok(config) => config,
        Err(error) => panic!("Unable to generate config: {}", error.message),
    };

    if !Path::new(config.working_path.as_str()).exists() {
        panic!(
            "WORKING_PATH directory does not exist: {}",
            config.working_path
        )
    }

    let tera = match Tera::new(format!("{}/templates/*.html", config.working_path).as_str()) {
        Ok(t) => t,
        Err(e) => panic!("Unable to generate config: {}", e),
    };

    let matches = Command::new("ohmyblog")
        .subcommand_required(true)
        .subcommand(Command::new("generate").about("generate all or specific files"))
        .subcommand(Command::new("webserver").about("starts the webserver"))
        .get_matches();

    if let Some(_) = matches.subcommand_matches("generate") {
        if let Err(e) = generate_all(&config, &tera) {
            panic!("Unable to generate file: {:?}", e.message)
        }
        return;
    }

    if let Some(_) = matches.subcommand_matches("webserver") {
        webserver(config).await;
    }
}

async fn webserver(config: Config) {
    let working_path = config.working_path.clone();
    if !Path::new(working_path.as_str()).exists() {
        eprintln!(
            "working path \"{}\" could not be found",
            config.working_path
        );
        process::exit(1);
    }

    let mut app = tide::with_state(config);
    if let Err(e) = app.at("/").serve_dir(working_path) {
        eprintln!("error on serve_dir: {}", e)
    }
    app.with(AuthMiddleware {});
    app.at("/api/files").get(ctrl_get_files);
    app.at("/api/changes").get(ctrl_get_changes);
    app.at("/api/preview").post(ctrl_get_preview);
    app.at("/api/file/new").post(ctrl_new_file);
    app.at("/api/folder/new").post(ctrl_new_folder);
    app.at("/api/stage").post(ctrl_stage);
    app.at("/api/revert").post(ctrl_revert);
    app.at("/api/upload").post(ctrl_upload);
    app.at("/api/save").post(ctrl_save);
    app.at("/api/rename").post(ctrl_rename);
    app.at("/api/delete").post(ctrl_delete);
    app.at("/api/commit").post(ctrl_commit);
    app.at("/api/generate").post(ctrl_generate);
    app.at("/api/push_remote").post(ctrl_push_remote);
    app.at("/api/pull_remote").post(ctrl_pull_remote);

    let listen = env::var("LISTEN").unwrap_or(String::from("127.0.0.1:8080"));
    let tide_cert_path = env::var("TIDE_CERT_PATH").unwrap_or(String::from(""));
    let tide_key_path = env::var("TIDE_KEY_PATH").unwrap_or(String::from(""));

    if tide_cert_path.len() > 0 && tide_key_path.len() > 0 {
        if let Err(e) = app.listen(TlsListener::build()
                                       .addrs(listen)
                                       .cert(tide_cert_path)
                                       .key(tide_key_path),
            ).await {
            eprintln!("unable to start webserver: {}", e)
        }
    } else {
        if let Err(e) = app.listen(listen).await {
            eprintln!("unable to start webserver: {}", e)
        }
    }
}
