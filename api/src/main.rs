mod blog;

use actix_files::Files;
use actix_web::dev::Server;
use actix_web::middleware::from_fn;
use actix_web::{web, App, HttpServer};
use clap::Command;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::env;
use std::path::Path;
use std::process;
use tera::Tera;

use crate::blog::auth_middleware::auth_middleware;

use crate::blog::config::Config;
use crate::blog::ctrl_commit::ctrl_commit;
use crate::blog::ctrl_delete::ctrl_delete;
use crate::blog::ctrl_generate::ctrl_generate;
use crate::blog::ctrl_get_changes::ctrl_get_changes;
use crate::blog::ctrl_get_files::ctrl_get_files;
use crate::blog::ctrl_get_preview::ctrl_get_preview;
use crate::blog::ctrl_get_attributes::ctrl_get_attributes;
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

#[actix_web::main]
async fn main() {
    let config = match Config::new() {
        Ok(config) => config,
        Err(e) => panic!("Unable to generate config: {}", e),
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
            panic!("Unable to generate file: {:?}", e)
        }
        return;
    }

    if let Some(_) = matches.subcommand_matches("webserver") {
        match run_web_server(config) {
            Ok(server) => {
                _ = server.await;
            }
            Err(e) => panic!("Unable to start webserver: {}", e),
        }
    }
}

fn run_web_server(config: Config) -> Result<Server, std::io::Error> {
    let working_path = config.working_path.clone();
    if !Path::new(working_path.as_str()).exists() {
        eprintln!(
            "working path \"{}\" could not be found",
            config.working_path
        );
        process::exit(1);
    }

    let runtime_data = web::Data::new(config);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(runtime_data.clone())
            .route("/api/files", web::get().to(ctrl_get_files))
            .route("/api/attributes", web::get().to(ctrl_get_attributes))
            .route("/api/changes", web::get().to(ctrl_get_changes))
            .route("/api/preview", web::post().to(ctrl_get_preview))
            .route("/api/file/new", web::post().to(ctrl_new_file))
            .route("/api/folder/new", web::post().to(ctrl_new_folder))
            .route("/api/stage", web::post().to(ctrl_stage))
            .route("/api/revert", web::post().to(ctrl_revert))
            .route("/api/upload", web::post().to(ctrl_upload))
            .route("/api/save", web::post().to(ctrl_save))
            .route("/api/rename", web::post().to(ctrl_rename))
            .route("/api/delete", web::post().to(ctrl_delete))
            .route("/api/commit", web::post().to(ctrl_commit))
            .route("/api/generate", web::post().to(ctrl_generate))
            .route("/api/push_remote", web::post().to(ctrl_push_remote))
            .route("/api/pull_remote", web::post().to(ctrl_pull_remote))
            .wrap(from_fn(auth_middleware))
            .service(
                Files::new("/", runtime_data.working_path.as_str())
                    .index_file("index.html")
                    .prefer_utf8(true),
            )
    });

    let listen = env::var("LISTEN").unwrap_or(String::from("127.0.0.1:8080"));
    let ssl_cert_path = env::var("SSL_CERT_PATH").unwrap_or(String::from(""));
    let ssl_key_path = env::var("SSL_KEY_PATH").unwrap_or(String::from(""));

    if ssl_cert_path.len() > 0 && ssl_key_path.len() > 0 {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
        builder.set_private_key_file(ssl_key_path, SslFiletype::PEM)?;
        builder.set_certificate_chain_file(ssl_cert_path)?;

        Ok(server.bind_openssl(listen, builder)?.run())
    } else {
        Ok(server.bind(listen)?.run())
    }
}
