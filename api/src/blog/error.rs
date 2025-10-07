use std::fmt;
use std::fmt::Display;
use tide::Response;
use tide::{Body, StatusCode};

#[derive(Debug, Clone)]
pub struct GeneratorError {
    pub message: String,
}

impl Display for GeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl GeneratorError {
    pub fn new(message: String) -> GeneratorError {
        return GeneratorError {
            message,
        };
    }
}

pub fn http_error(status: StatusCode, body: impl Into<Body>) -> Response {
    let response = Response::builder(status)
        .body(body)
        .build();
    return response;
}