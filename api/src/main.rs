mod blog;

use std::env;
use clap::Command;
use std::path::Path;
use std::process;
use tera::Tera;
use tide::http::headers::HeaderValue;

use tide::security::{CorsMiddleware, Origin};
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
    if !Path::new(config.working_path.as_str()).exists() {
        eprintln!(
            "working path \"{}\" could not be found",
            config.working_path
        );
        process::exit(1);
    }

    //let cors = CorsMiddleware::new().allow_origin(Origin::from("*"));
    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    let mut app = tide::with_state(config);
    app.with(cors);
    app.with(AuthMiddleware {});
    app.at("/files").get(ctrl_get_files);
    app.at("/changes").get(ctrl_get_changes);
    app.at("/preview").post(ctrl_get_preview);
    app.at("/file/new").post(ctrl_new_file);
    app.at("/folder/new").post(ctrl_new_folder);
    app.at("/stage").post(ctrl_stage);
    app.at("/revert").post(ctrl_revert);
    app.at("/upload").post(ctrl_upload);
    app.at("/save").post(ctrl_save);
    app.at("/rename").post(ctrl_rename);
    app.at("/delete").post(ctrl_delete);
    app.at("/commit").post(ctrl_commit);
    app.at("/generate").post(ctrl_generate);
    app.at("/push_remote").post(ctrl_push_remote);

    let listen = env::var("LISTEN").unwrap_or(String::from("127.0.0.1:8080"));
    let tide_cert_path = env::var("TIDE_CERT_PATH").unwrap_or(String::from(""));
    let tide_key_path = env::var("TIDE_KEY_PATH").unwrap_or(String::from(""));

    if tide_cert_path.len() > 0 && tide_key_path.len() > 0 {
        if let Err(e) = app.listen(TlsListener::build()
                                       .addrs(listen)
                                       .cert(tide_cert_path)
                                       .key(tide_key_path),
            ).await {
            eprintln!("{}", e)
        }
    } else {
        if let Err(e) = app.listen(listen).await {
            eprintln!("{}", e)
        }
    }
}
