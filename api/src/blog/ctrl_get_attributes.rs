use crate::blog::generator::Generator;
use crate::Config;
use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use tera::Tera;

pub async fn ctrl_get_attributes(runtime: web::Data<Config>) -> actix_web::Result<impl Responder> {
    let tera = match Tera::new(format!("{}/templates/*.html", runtime.working_path).as_str()) {
        Ok(t) => t,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "unable to generate config: {}",
                e
            )));
        }
    };

    let mut generator = Generator::new(
        &tera,
        runtime.get_input_path(),
        runtime.get_output_path(),
        None,
    );

    let attributes = match generator.get_attributes() {
        Ok(attributes) => attributes,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
    };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(json!(attributes).to_string()))
}
