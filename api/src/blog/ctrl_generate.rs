use comrak::plugins::syntect::SyntectAdapter;
use tera::Tera;
use tide::{Request, Response};
use crate::blog::config::{Config, HIGHLIGHT_THEME};
use crate::blog::generator::Generator;

pub async fn ctrl_generate(req: Request<Config>) -> tide::Result {
    let tera = match Tera::new(format!("{}/templates/*.html", req.state().working_path).as_str()) {
        Ok(t) => t,
        Err(e) => panic!("Unable to generate config: {:?}", e),
    };

    let adapter = SyntectAdapter::new(HIGHLIGHT_THEME);
    let mut generator = Generator::new(
        &tera,
        req.state().get_input_path(),
        req.state().get_output_path(),
        Some(&adapter),
    );
    generator.log_to_buffer();

    match generator.generate() {
        Ok(_) => {}
        Err(e) => panic!("Unable to generate file: {:?}", e.message),
    }

    let response = Response::builder(200)
        .body(generator.get_log_result())
        .build();
    Ok(response)
}