use tide::{Middleware, Next, Request, Response, StatusCode};
use tide::http::mime;
use crate::blog::config::ConfigType;

const AUTH_HEADER_NAME: &str = "Authorization";

pub struct AuthMiddleware {}

#[async_trait::async_trait]
impl<State> Middleware<State> for AuthMiddleware
    where
        State: Clone + Send + Sync + ConfigType + 'static,
{
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        if !req.url().path().starts_with("/api/") {
            return Ok(next.run(req).await);
        }

        let auth_header = req.header(AUTH_HEADER_NAME);
        if auth_header.is_none() {
            return Ok(unauthorized("no auth header"));
        }
        let header_value: Vec<_> = auth_header.unwrap().into_iter().collect();

        if header_value.is_empty() {
            return Ok(unauthorized("empty auth header"));
        }

        if header_value.len() > 1 {
            return Ok(unauthorized("multiple auth headers"));
        }

        let value = header_value.get(0).unwrap().to_string();
        if !value.starts_with("Token ") {
            return Ok(unauthorized("invalid token type"));
        }

        let token = value.replace("Token ", "");
        if token != req.state().get_token() {
            return Ok(unauthorized("invalid token"));
        }

        Ok(next.run(req).await)
    }
}

fn unauthorized(message: &str) -> Response {
    return Response::builder(StatusCode::Unauthorized)
        //.header("Access-Control-Allow-Origin", "*")
        .body(message)
        .content_type(mime::PLAIN)
        .build();
}