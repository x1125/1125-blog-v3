use crate::blog::config::{Config, HIGHLIGHT_THEME};
use crate::blog::error::http_error;
use crate::blog::generator::Generator;
use comrak::plugins::syntect::SyntectAdapter;
use tera::Tera;
use tide::{Request, Response, StatusCode};

pub async fn ctrl_generate(req: Request<Config>) -> tide::Result {
    let tera = match Tera::new(format!("{}/templates/*.html", req.state().working_path).as_str()) {
        Ok(t) => t,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("unable to generate config: {}", e)));
        }
    };

    let adapter = SyntectAdapter::new(Some(HIGHLIGHT_THEME));
    let mut generator = Generator::new(
        &tera,
        req.state().get_input_path(),
        req.state().get_output_path(),
        Some(&adapter),
    );
    generator.log_to_buffer();

    if let Err(e) = generator.generate() {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to generate file: {}", e.message)));
    }

    Ok(Response::builder(StatusCode::Ok)
        .body(generator.get_log_result())
        .build())
}