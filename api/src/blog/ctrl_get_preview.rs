use crate::blog::config::{Config, HIGHLIGHT_THEME};
use crate::blog::error::http_error;
use crate::blog::generator::{Generator, Post};
use comrak::plugins::syntect::SyntectAdapter;
use tera::Tera;
use tide::http::mime;
use tide::prelude::*;
use tide::{Request, Response, StatusCode};

#[derive(Debug, Deserialize)]
struct PreviewData {
    content: String,
}

pub async fn ctrl_get_preview(mut req: Request<Config>) -> tide::Result {
    let PreviewData { content } = req.body_json().await?;

    let tera = match Tera::new(format!("{}/templates/*.html", req.state().working_path).as_str()) {
        Ok(t) => t,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("unable to generate config: {:?}", e)));
        }
    };

    let adapter = SyntectAdapter::new(Some(HIGHLIGHT_THEME));
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
            return Ok(http_error(StatusCode::InternalServerError, format!(
                "unable to generate post preview: {}", e)));
        }
    };

    let html = match generator.generate_preview(&mut content_mut) {
        Ok(html) => html,
        Err(e) => {
            return Ok(http_error(StatusCode::InternalServerError, format!("unable to generate preview: {}", e.message)));
        }
    };

    let posts: Vec<Post> = vec![post];
    generator.generate_preview_images(&posts);
    if let Err(e) = generator.remove_exif_data(&posts) {
        return Ok(http_error(StatusCode::InternalServerError, format!("unable to remove exif data: {}", e.message)));
    }

    Ok(Response::builder(StatusCode::Ok)
        .body(html)
        .content_type(mime::HTML)
        .build())
}