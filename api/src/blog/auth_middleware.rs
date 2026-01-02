use crate::blog::config::Config;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::ErrorUnauthorized;
use actix_web::middleware::Next;
use actix_web::web::Data;
use actix_web::Error;

pub(crate) async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if !req.path().starts_with("/api/") {
        return next.call(req).await;
    }

    let config: &Data<Config> = match req.app_data() {
        Some(config) => config,
        None => {
            return Err(ErrorUnauthorized("missing runtime"));
        }
    };

    let api_key = config.token.clone();
    let req_api_key: String = match req.headers().get("Authorization") {
        Some(api_key) => api_key.to_str().unwrap().replace("Token ", "").to_string(),
        None => String::new(),
    };

    if api_key != req_api_key {
        return Err(ErrorUnauthorized("missing/wrong token"));
    }
    next.call(req).await
}
