use crate::blog::config::{Config, HIGHLIGHT_THEME};
use crate::blog::generator::Generator;
use actix_web::{web, Responder};
use comrak::plugins::syntect::SyntectAdapter;
use tera::Tera;

pub async fn ctrl_generate(runtime: web::Data<Config>) -> actix_web::Result<impl Responder> {
    let tera = match Tera::new(format!("{}/templates/*.html", runtime.working_path).as_str()) {
        Ok(t) => t,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "unable to generate config: {}",
                e
            )));
        }
    };

    let adapter = SyntectAdapter::new(Some(HIGHLIGHT_THEME));
    let mut generator = Generator::new(
        &tera,
        runtime.get_input_path(),
        runtime.get_output_path(),
        Some(&adapter),
    );
    generator.log_to_buffer();

    if let Err(e) = generator.generate() {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to generate file: {}",
            e
        )));
    }

    Ok(generator.get_log_result())
}
