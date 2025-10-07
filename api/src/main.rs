use std::process;
use clap::Command;
use tera::Tera;

use tide::security::{CorsMiddleware, Origin};

use lib::config::Config;
use crate::blog::ctrl_get_admin_js::ctrl_get_admin_js;
use crate::blog::ctrl_get_state::ctrl_get_state;
use crate::lib::generator::generate_all;

mod blog;
mod lib;

#[async_std::main]
async fn main() {
    let config = match Config::new() {
        Ok(config) => config,
        Err(error) => panic!("Unable to generate config: {:?}", error.message),
    };

    if !config.working_path.exists() {
        panic!("WORKING_PATH directory does not exist: {:?}", config.working_path.to_string_lossy())
    }

    let tera = match Tera::new(format!("{}/templates/*.html", config.working_path.to_string_lossy()).as_str()) {
        Ok(t) => t,
        Err(e) => panic!("Unable to generate config: {:?}", e)
    };

    let matches = Command::new("ohmyblog")
        .subcommand_required(true)
        .subcommand(
            Command::new("generate")
                .about("generate all or specific files")
        )
        .subcommand(
            Command::new("webserver")
                .about("starts the webserver")
        )
        .get_matches();

    if let Some(_) = matches.subcommand_matches("generate") {
        match generate_all(config, &tera) {
            Ok(_) => {},
            Err(e) => panic!("Unable to generate file: {:?}", e.message)
        }
        return;
    }

    if let Some(_) = matches.subcommand_matches("webserver") {
        webserver(config).await;
    }
}

async fn webserver(config: Config) {
    if !config.working_path.exists() {
        eprintln!("working path \"{}\" could not be found", config.working_path.to_string_lossy());
        process::exit(1);
    }

    let cors = CorsMiddleware::new()
        .allow_origin(Origin::from("*"));

    let mut app = tide::with_state(config);
    app.with(cors);
    app.at("/admin.js").get(ctrl_get_admin_js);
    //app.at("/state").with(AuthMiddleware {}).get(get_get_state);
    app.at("/state").get(ctrl_get_state);
    match app.listen("127.0.0.1:8080").await {
        Ok(()) => {}
        Err(err) => eprintln!("{}", err.to_string())
    }
}