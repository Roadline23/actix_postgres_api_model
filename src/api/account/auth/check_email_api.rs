use crate::{
    error::bad_request,
    types::auth::check_email::CheckEmailDataRequest,
    utils::{
        cookie_utils::{create_cookie, CookiePayload},
        jwt_utils::decode_token,
        time_utils::MAX_AGE_1H_TEST,
        two_factors_auth_utils::TwoFactorsAuth,
    },
};
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use chrono::Utc;
use sea_orm::DatabaseConnection;
use serde_json::json;
use service::query::user_queries::UserQuery;
use tracing::error;

#[post("/checkemail")]
pub async fn check_email(
    db: Data<DatabaseConnection>,
    body: Json<CheckEmailDataRequest>,
) -> HttpResponse {
    let token = match body.token.to_owned() {
        Some(t) => t,
        None => {
            error!("Cannot get token on check email");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let decrypted_token = match decode_token(token.as_str()) {
        Ok(tk) => tk,
        Err(resp_error) => return HttpResponse::BadRequest().json(resp_error),
    };

    let user = match UserQuery::find_user_by_id(&db, decrypted_token.payload).await {
        Ok(u) => u,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let two_fa = match UserQuery::find_related_two_fa(&db, &user).await {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let resp_check_deadline = TwoFactorsAuth::check_deadline(&two_fa);

    if resp_check_deadline.still_time {
        return bad_request(
            "Auth",
            "TimeLeft",
            Some(json!({
                "timeLeft": resp_check_deadline.time_left,
            })),
        );
    } else {
        match TwoFactorsAuth::reset_tries(&two_fa, &db).await {
            Ok(_) => (),
            Err(_) => return HttpResponse::InternalServerError().finish(),
        }

        let expires_at = Utc::now().timestamp() + MAX_AGE_1H_TEST;
        let token = CookiePayload {
            id: user.id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        return HttpResponse::Ok().cookie(cookie).finish();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::resp_errors::RespErrors,
        utils::{jwt_utils::create_token, time_utils::MAX_AGE_3M},
    };
    use actix_web::{
        test,
        web::{self, Data},
        App,
    };
    use chrono::Utc;
    use entity::entities::{two_fa_entity::two_fa_model, user_entity::user_model};
    use reqwest::StatusCode;
    use sea_orm::{DatabaseBackend, DatabaseConnection, DbErr, MockDatabase, MockExecResult, RuntimeErr};
    use serde_json::Value;
    use uuid::Uuid;

    use super::{check_email, CheckEmailDataRequest};

    fn mock_db_to_verified_email() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[user_model::Model {
                id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                f: String::from("Rob"),
                l: String::from("Doe"),
                e: String::from("test.pro.1@gmail.com"),
                ph: String::from("0600000001"),
                t: true,
                pv: true,
                ..Default::default()
            }]])
            .append_query_results([[two_fa_model::Model {
                id: 1,
                v_e: false,
                t: 3,
                s: 0,
                c: None,
                up: None,
                ex: 0,
                v_ph: false,
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            }]])
            .append_exec_results([MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .into_connection()
    }

    fn mock_db_user_not_found() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors(vec![DbErr::RecordNotFound("user not found".to_string())])
            .into_connection()
    }

    fn mock_db_two_fa_not_found() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[user_model::Model {
                id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                f: String::from("Rob"),
                l: String::from("Doe"),
                e: String::from("test.pro.1@gmail.com"),
                ph: String::from("0600000001"),
                t: true,
                pv: true,
                ..Default::default()
            }]])
            .append_query_errors(vec![DbErr::RecordNotFound("two_fa not found".to_string())])
            .into_connection()
    }

    fn mock_db_with_blocked_account() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[user_model::Model {
                id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                f: String::from("Rob"),
                l: String::from("Doe"),
                e: String::from("test.pro.1@gmail.com"),
                ph: String::from("0600000001"),
                t: true,
                pv: true,
                ..Default::default()
            }]])
            .append_query_results([[two_fa_model::Model {
                id: 1,
                v_e: false,
                t: 3,
                s: 0,
                c: None,
                up: Some((Utc::now().timestamp_millis() as i64) + 300_000), // 5 minutes left
                ex: 0,
                v_ph: false,
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            }]])
            .into_connection()
    }

    fn mock_db_to_cannot_reset_tries() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[user_model::Model {
                id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                f: String::from("Rob"),
                l: String::from("Doe"),
                e: String::from("test.pro.1@gmail.com"),
                ph: String::from("0600000001"),
                t: true,
                pv: true,
                ..Default::default()
            }]])
            .append_query_results([[two_fa_model::Model {
                id: 1,
                v_e: false,
                t: 0,
                s: 2,
                c: None,
                up: None,
                ex: 0,
                v_ph: false,
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            }]])
            .append_query_errors(vec![DbErr::Query(RuntimeErr::Internal(
                "Failed to reset tries".to_string(),
            ))])
            .into_connection()
    }

    #[actix_web::test]
    async fn test_checkout_email_success() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_to_verified_email());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_email)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let req_data = CheckEmailDataRequest {
            token: Some(create_token(&user_id, Utc::now().timestamp() + MAX_AGE_3M)),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkemail")
            .set_json(&req_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let cookie = resp.response().cookies().next().unwrap();

        assert_eq!(cookie.name(), "token");
    }

    #[actix_web::test]
    async fn test_checkout_body_is_empty() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_to_verified_email());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_email)),
        )
        .await;

        let req_data = CheckEmailDataRequest {
            token: Some(String::from("invalid_token")),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkemail")
            .set_json(&req_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_unknown_user_on_token() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_user_not_found());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_email)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let req_data = CheckEmailDataRequest {
            token: Some(create_token(&user_id, Utc::now().timestamp() + MAX_AGE_3M)),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkemail")
            .set_json(&req_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_two_fa_not_found() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_two_fa_not_found());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_email)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let req_data = CheckEmailDataRequest {
            token: Some(create_token(&user_id, Utc::now().timestamp() + MAX_AGE_3M)),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkemail")
            .set_json(&req_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_account_blocked() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_with_blocked_account());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_email)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let req_data = CheckEmailDataRequest {
            token: Some(create_token(&user_id, Utc::now().timestamp() + MAX_AGE_3M)),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkemail")
            .set_json(&req_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;

        let resp_errors: RespErrors<Value> = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp_errors.kind, "Auth");
        assert_eq!(resp_errors.reason, "TimeLeft");
    }

    #[actix_web::test]
    async fn test_cannot_reset_tries() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_to_cannot_reset_tries());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_email)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let req_data = CheckEmailDataRequest {
            token: Some(create_token(&user_id, Utc::now().timestamp() + MAX_AGE_3M)),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkemail")
            .set_json(&req_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
