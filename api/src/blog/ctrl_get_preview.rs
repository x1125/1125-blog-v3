use crate::blog::config::{Config, HIGHLIGHT_THEME};
use crate::blog::generator::{Generator, Post};
use comrak::plugins::syntect::SyntectAdapter;
use tera::Tera;
use tide::http::mime;
use tide::prelude::*;
use tide::{Request, Response};

#[derive(Debug, Deserialize)]
struct PreviewData {
    content: String,
}

fn http_error(msg: String) -> Result<tide::Response, tide::Error> {
    return Ok(Response::builder(500).body(msg).build());
}

pub async fn ctrl_get_preview(mut req: Request<Config>) -> tide::Result {
    let PreviewData { content } = req.body_json().await?;

    let tera = match Tera::new(format!("{}/templates/*.html", req.state().working_path).as_str()) {
        Ok(t) => t,
        Err(e) => {
            return http_error(format!("Unable to generate config: {:?}", e));
        }
    };

    let adapter = SyntectAdapter::new(HIGHLIGHT_THEME);
    let mut generator = Generator::new(
        &tera,
        req.state().get_input_path(),
        req.state().get_output_path(),
        Some(&adapter),
    );
    let mut content_mut = content.clone();

    let post = match generator.new_post(String::from("preview"), &mut content_mut) {
        Ok(post) => post,
        Err(e) => {
            return http_error(format!(
                "unable to generate post preview: {}",
                e.to_string()
            ));
        }
    };

    let html = match generator.generate_preview(&mut content_mut) {
        Ok(html) => html,
        Err(e) => {
            return http_error(e.message);
        }
    };

    let posts: Vec<Post> = vec![post];
    generator.generate_preview_images(&posts);
    let e = generator.remove_exif_data(&posts);
    if e.is_err() {
        return http_error(e.unwrap_err().message);
    }

    let response = Response::builder(200)
        .body(html)
        .content_type(mime::HTML)
        .build();
    Ok(response)
}
