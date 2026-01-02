use crate::blog::config::{Config, HIGHLIGHT_THEME};
use crate::blog::generator::{Generator, Post};
use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse, Responder};
use comrak::plugins::syntect::SyntectAdapter;
use serde::Deserialize;
use tera::Tera;

#[derive(Deserialize)]
pub struct PreviewData {
    content: String,
}

pub async fn ctrl_get_preview(
    runtime: web::Data<Config>,
    preview_data: web::Json<PreviewData>,
) -> actix_web::Result<impl Responder> {
    let tera = match Tera::new(format!("{}/templates/*.html", runtime.working_path).as_str()) {
        Ok(t) => t,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "unable to generate config: {:?}",
                e
            )))
        }
    };

    let adapter = SyntectAdapter::new(Some(HIGHLIGHT_THEME));
    let mut generator = Generator::new(
        &tera,
        runtime.get_input_path(),
        runtime.get_output_path(),
        Some(&adapter),
    );
    let mut content_mut = preview_data.content.clone();

    let post = match generator.new_post(String::from("preview"), &mut content_mut) {
        Ok(post) => post,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "unable to generate post preview: {}",
                e
            )))
        }
    };

    let html = match generator.generate_preview(&mut content_mut) {
        Ok(html) => html,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "unable to generate preview: {}",
                e
            )))
        }
    };

    let posts: Vec<Post> = vec![post];
    if let Err(e) = generator.generate_preview_images(&posts) {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to generate preview images: {}",
            e
        )));
    }
    if let Err(e) = generator.remove_exif_data(&posts) {
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "unable to remove exif data: {}",
            e
        )));
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html))
}
