use actix_web::{
    cookie::{time::OffsetDateTime, Cookie, SameSite},
    HttpRequest, dev::ServiceRequest,
};
use sea_orm::prelude::Uuid;
use tracing::error;

use crate::utils::crypto_utils::encrypt_payload;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CookiePayload {
    pub exp_at: i64,
    pub id: Uuid,
}

pub fn create_cookie<'a>(name: &'a str, payload: &'a CookiePayload) -> Cookie<'a> {
    let encrypted_payload = encrypt_payload(&payload).expect("Failed to encrypt token");
    let offset_date_time = OffsetDateTime::from_unix_timestamp(payload.exp_at).unwrap();
    let r = serde_json::json!(encrypted_payload).to_string();
    return Cookie::build(name, r)
        .domain("localhost")
        .path("/")
        .http_only(true)
        .expires(offset_date_time)
        .secure(false) // set to true if you're using HTTPS
        .same_site(SameSite::Strict)
        .finish();
}

pub fn get_cookie_from_service_request(req: &ServiceRequest, name: &str) -> Option<String> {
    match req.cookie(name) {
        Some(c) => {
            let parts_to_string = c.to_string();
            let parts: Vec<&str> = parts_to_string.split("=").collect();
            if parts.len() > 1 {
                return Some(parts[1].to_owned());
            } else {
                error!("Cannot get cookie on auth_middleware");
                return None;
            }
        }
        None => {
            error!("Cannot get cookie on auth_middleware");
            return None;
        }
    };
}

pub fn get_cookie_from_http_request(req: &HttpRequest, name: &str) -> Option<String> {
    match req.cookie(name) {
        Some(c) => {
            let parts_to_string = c.to_string();
            let parts: Vec<&str> = parts_to_string.split("=").collect();
            if parts.len() > 1 {
                return Some(parts[1].to_owned());
            } else {
                error!("Cannot get cookie on auth endpoints");
                return None;
            }
        }
        None => {
            error!("Cannot get cookie on auth endpoints");
            return None;
        }
    };
}

pub fn delete_cookie<'a>(name: &'a str) -> Cookie<'a> {
    return Cookie::build(name, "")
        .domain("localhost")
        .path("/")
        .http_only(true)
        .expires(OffsetDateTime::now_utc())
        .secure(false) // set to true if you're using HTTPS
        .same_site(SameSite::Strict)
        .finish();
}
