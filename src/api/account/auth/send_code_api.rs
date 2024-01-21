use crate::{
    error::{bad_request, internal_server_error},
    utils::{
        cookie_utils::{create_cookie, CookiePayload, get_cookie_from_http_request},
        crypto_utils::{decrypt_payload, EncryptedPayload},
        time_utils::MAX_AGE_3M,
        two_factors_auth_utils::{SendingState, TwoFactorsAuth},
    },
};
use actix_web::{post, web::Data, HttpRequest, HttpResponse};
use chrono::Utc;
use sea_orm::DatabaseConnection;
use serde_json::json;
use service::query::user_queries::UserQuery;

#[post("/sendcode")]
pub async fn send_code(req: HttpRequest, db: Data<DatabaseConnection>) -> HttpResponse {
    let cookie = match get_cookie_from_http_request(&req, "token") {
        Some(c) => c,
        None => {
            return HttpResponse::InternalServerError().finish();
        }
    };

    let token = serde_json::from_str::<EncryptedPayload>(&cookie.as_str()).unwrap();

    let cookie_payload: CookiePayload = match decrypt_payload(&token.order, &token.content) {
        Ok(p) => p,
        Err(_) => {
            return HttpResponse::InternalServerError().finish();
        }
    };

    if cookie_payload.exp_at < Utc::now().timestamp() {
        return bad_request::<String>("Token", "Expired", None);
    }

    let user = match UserQuery::find_user_by_id(&db, cookie_payload.id).await {
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
        let code = TwoFactorsAuth::generate_code(&two_fa);
        TwoFactorsAuth::update_two_fa_with_new_code(&two_fa, &code, &db).await;
        let send_code = TwoFactorsAuth::send_code_to_pro(&two_fa, user.to_owned(), &code).await;

        match send_code {
            SendingState::Sent => {
                let num_of_sending =
                    TwoFactorsAuth::update_two_fa_with_new_num_of_sending(&two_fa, &db).await;

                let token = CookiePayload {
                    id: user.id,
                    exp_at: Utc::now().timestamp() + MAX_AGE_3M,
                };

                let cookie = create_cookie("token", &token);

                return HttpResponse::Ok().cookie(cookie).json(json!({
                    "sent": num_of_sending,
                }));
            }
            SendingState::AlreadySent => {
                let time_left = TwoFactorsAuth::block_account(&two_fa, &db).await;
                return bad_request(
                    "Auth",
                    "TimeLeft",
                    Some(json!({
                        "timeLeft": time_left,
                    })),
                );
            }
            SendingState::NotSent => {
                return internal_server_error::<String>("SMS", "NotSent", None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::resp_errors::RespErrors,
        utils::{
            cookie_utils::{create_cookie, CookiePayload},
            time_utils::MAX_AGE_3M,
        },
    };
    use actix_web::{
        test,
        web::{self, Data},
        App,
    };
    use chrono::Utc;
    use entity::entities::{two_fa_entity::two_fa_model, user_entity::user_model};
    use reqwest::StatusCode;
    use sea_orm::{DatabaseBackend, DatabaseConnection, DbErr, MockDatabase};
    use serde_json::Value;
    use uuid::Uuid;

    use super::send_code;

    fn mock_db_to_sending_code() -> DatabaseConnection {
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

    fn mock_db_with_sent_code_already() -> DatabaseConnection {
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
                s: 2, // 2 sent code already
                c: None,
                up: None,
                ex: 0,
                v_ph: false,
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            }]])
            .into_connection()
    }

    #[actix_web::test]
    async fn test_sending_code_success() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_to_sending_code());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(send_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let req = test::TestRequest::post()
            .uri("/api/sendcode")
            .cookie(cookie)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let cookie = resp.response().cookies().next().unwrap();

        assert_eq!(cookie.name(), "token");

        let body = test::read_body(resp).await;

        let number_of_sending = serde_json::from_slice::<Value>(&body).unwrap();

        assert_eq!(number_of_sending["sent"], 1);
    }

    #[actix_web::test]
    async fn test_sending_code_no_cookie() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_to_sending_code());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(send_code)),
        )
        .await;

        let req = test::TestRequest::post().uri("/api/sendcode").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_sending_code_expired_cookie() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_to_sending_code());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(send_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() - MAX_AGE_3M;
        let expired_token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let expired_cookie = create_cookie("token", &expired_token);

        let req = test::TestRequest::post()
            .uri("/api/sendcode")
            .cookie(expired_cookie)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;

        let resp_errors: RespErrors<Value> = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp_errors.kind, "Token");
        assert_eq!(resp_errors.reason, "Expired");
    }

    #[actix_web::test]
    async fn test_sending_code_user_not_found() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_user_not_found());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(send_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let req = test::TestRequest::post()
            .uri("/api/sendcode")
            .cookie(cookie)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_sending_code_two_fa_not_found() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_two_fa_not_found());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(send_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let req = test::TestRequest::post()
            .uri("/api/sendcode")
            .cookie(cookie)
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
                .service(web::scope("/api").service(send_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let req = test::TestRequest::post()
            .uri("/api/sendcode")
            .cookie(cookie)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;

        let resp_errors: RespErrors<Value> = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp_errors.kind, "Auth");
        assert_eq!(resp_errors.reason, "TimeLeft");
    }

    #[actix_web::test]
    async fn test_sending_code_already_sent_error() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_with_sent_code_already());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(send_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let req = test::TestRequest::post()
            .uri("/api/sendcode")
            .cookie(cookie)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;

        let resp_errors: RespErrors<Value> = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp_errors.kind, "Auth");
        assert_eq!(resp_errors.reason, "TimeLeft");
    }
}
