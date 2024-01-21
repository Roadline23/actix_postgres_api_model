use actix_web::{http::header, HttpResponse};

use self::resp_errors::RespErrors;

pub mod errors;
pub mod messages;
pub mod resp_errors;
pub mod custom_error;

pub fn ok_response<T>(resp: Option<T>) -> HttpResponse
where
    T: serde::Serialize,
{
    match resp {
        Some(s) => return HttpResponse::Ok().json(s),
        None => {
            return HttpResponse::Ok()
                .insert_header(header::ContentType::plaintext())
                .finish()
        }
    }
}

pub fn bad_request<T>(kind: &str, reason: &str, errors: Option<T>) -> HttpResponse
where
    T: serde::Serialize,
{
    return HttpResponse::BadRequest().json(RespErrors::new(kind, reason, errors));
}

pub fn internal_server_error<T>(kind: &str, reason: &str, errors: Option<T>) -> HttpResponse
where
    T: serde::Serialize,
{
    return HttpResponse::InternalServerError().json(RespErrors::new(kind, reason, errors));
}
