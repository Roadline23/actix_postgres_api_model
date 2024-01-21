use std::future::{ready, Ready};

use crate::utils::{
    cookie_utils::{get_cookie_from_service_request, CookiePayload},
    crypto_utils::{decrypt_payload, EncryptedPayload},
};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error, web, Error,
};
use chrono::Utc;
use futures_util::future::LocalBoxFuture;

use super::app_state::AppState;

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request = match chekout_valid_cookie(req) {
            Ok(authenticated_req) => authenticated_req,
            Err(err) => {
                return Box::pin(async { Err(err) });
            }
        };

        let fut = self.service.call(request);

        Box::pin(async move {
            let res = fut.await?;

            println!("Hi from response");
            Ok(res)
        })
    }
}

pub fn chekout_valid_cookie(req: ServiceRequest) -> Result<ServiceRequest, Error> {
    let cookie = match get_cookie_from_service_request(&req, "SESSIONID") {
        Some(t) => t,
        None => {
            return Err(error::ErrorGatewayTimeout("Token expired"));
        }
    };

    let token = serde_json::from_str::<EncryptedPayload>(&cookie.as_str()).unwrap();

    let cookie_payload: CookiePayload = match decrypt_payload(&token.order, &token.content) {
        Ok(p) => p,
        Err(_) => {
            return Err(error::ErrorGatewayTimeout("Token expired"));
        }
    };

    if cookie_payload.exp_at < Utc::now().timestamp() {
        return Err(error::ErrorGatewayTimeout("Token expired"));
    } else {
        req.app_data::<web::Data<AppState>>().map(|app_state| {
            let mut id_state = app_state.id.lock().unwrap();
            *id_state = Some(cookie_payload.id);
        });
        Ok(req)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use actix_web::{http, test, App, HttpResponse};
    use uuid::Uuid;

    use crate::utils::{cookie_utils::create_cookie, time_utils::MAX_AGE_3M};

    use super::*;
    #[actix_web::test]
    async fn test_auth_middleware_success() {
        let app_state = web::Data::new(AppState {
            id: Mutex::new(None),
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .wrap(Auth)
                .service(web::resource("/").to(|| HttpResponse::Ok())),
        )
        .await;

        let token = CookiePayload {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            exp_at: Utc::now().timestamp() + MAX_AGE_3M,
        };

        let cookie = create_cookie("SESSIONID", &token);

        let req = test::TestRequest::get()
            .uri("/")
            .cookie(cookie)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_fn_chekout_valid_cookie_success() {
        let token = CookiePayload {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            exp_at: Utc::now().timestamp() + MAX_AGE_3M,
        };

        let cookie = create_cookie("SESSIONID", &token);

        let req = test::TestRequest::default().cookie(cookie).to_srv_request();

        let srv = chekout_valid_cookie(req);

        assert!(srv.is_ok());
    }

    #[actix_web::test]
    async fn test_fn_chekout_valid_cookie_no_cookie() {
        let req = test::TestRequest::default().to_srv_request();

        let result = chekout_valid_cookie(req);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Cookie not found".to_string()
        );
    }

    #[actix_web::test]
    async fn test_fn_chekout_valid_cookie_expired_cookie() {
        let token = CookiePayload {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            exp_at: Utc::now().timestamp() - MAX_AGE_3M,
        };

        let cookie = create_cookie("SESSIONID", &token);

        let req = test::TestRequest::default().cookie(cookie).to_srv_request();

        let result = chekout_valid_cookie(req);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Token expired".to_string());
    }
}
