use crate::{
    error::bad_request,
    types::auth::check_code::CheckCodeRequest,
    utils::{
        cookie_utils::{create_cookie, delete_cookie, get_cookie_from_http_request, CookiePayload},
        crypto_utils::{decrypt_payload, EncryptedPayload},
        time_utils::MAX_AGE_2J,
        two_factors_auth_utils::TwoFactorsAuth,
    },
};
use actix_web::{
    post,
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use chrono::Utc;
use sea_orm::DatabaseConnection;
use serde_json::json;
use service::query::user_queries::UserQuery;

#[post("/checkcode")]
pub async fn check_code(
    req: HttpRequest,
    db: Data<DatabaseConnection>,
    body: Json<CheckCodeRequest>,
) -> HttpResponse {
    let cookie = match get_cookie_from_http_request(&req, "token") {
        Some(t) => t,
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
        let check_code = TwoFactorsAuth::check_code(&two_fa, &body.code);

        if check_code.valid {
            let account = json!({
                "firstName": user.f,
                "lastName": user.l,
            });

            let token = CookiePayload {
                id: user.id,
                exp_at: Utc::now().timestamp() + MAX_AGE_2J,
            };

            let session_cookie = create_cookie("SESSIONID", &token);

            return HttpResponse::Ok().cookie(session_cookie).json(json!({
                "connection test": "ok",
                "account": account,
            }));
        } else {
            let tries = TwoFactorsAuth::update_pro_by_remove_one_try(&two_fa, &db).await;
            if tries == 0 {
                let time_left = TwoFactorsAuth::block_account(&two_fa, &db).await;
                return bad_request(
                    "Auth",
                    "TimeLeft",
                    Some(json!({
                        "timeLeft": time_left,
                    })),
                );
            } else {
                return bad_request(
                    "Code",
                    "Invalid",
                    Some(json!({
                        "tries": tries,
                    })),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::resp_errors::RespErrors,
        types::auth::check_code::CheckCodeRequest,
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
    use entity::entities::{
        two_fa_entity::two_fa_model,
        user_entity::user_model,
    };
    use reqwest::StatusCode;
    use sea_orm::{DatabaseBackend, DatabaseConnection, DbErr, MockDatabase};
    use serde_json::{json, Value};
    use uuid::Uuid;

    use super::check_code;

    fn mock_db_to_checking_code() -> DatabaseConnection {
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
                v_e: true,
                t: 3,
                s: 1,
                c: Some(String::from("123456")),
                up: None,
                ex: 0,
                v_ph: false,
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            }]])
            .append_query_results([[pro_model::Model {
                id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                s: String::from("883116000"),
                cs: true,
                sa: true,
                den: Some(String::from("Rob Company")),
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
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

    fn mock_db_pro_not_found() -> DatabaseConnection {
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
                v_e: true,
                t: 3,
                s: 1,
                c: Some(String::from("123456")),
                up: None,
                ex: 0,
                v_ph: false,
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            }]])
            .append_query_errors(vec![DbErr::RecordNotFound("Pro not found".to_string())])
            .into_connection()
    }

    fn mock_db_invalid_code() -> DatabaseConnection {
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
                v_e: true,
                t: 2,
                s: 1,
                c: Some(String::from("123456")),
                up: None,
                ex: 0,
                v_ph: false,
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            }]])
            .into_connection()
    }

    fn mock_db_too_many_attempts() -> DatabaseConnection {
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
                v_e: true,
                t: 1,
                s: 1,
                c: Some(String::from("123456")),
                up: None,
                ex: 0,
                v_ph: false,
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            }]])
            .into_connection()
    }

    #[actix_web::test]
    async fn test_checkout_code_success() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_to_checking_code());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let req_data = CheckCodeRequest {
            code: String::from("123456"),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkcode")
            .set_json(&req_data)
            .cookie(cookie)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let cookie = resp.response().cookies().next().unwrap();

        assert_eq!(cookie.name(), "SESSIONID");

        let body = test::read_body(resp).await;

        let resp_body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp_body["connection test"], "ok");
        assert_eq!(resp_body["account"]["firstName"], "Rob");
        assert_eq!(resp_body["account"]["tva"], false);

        assert_eq!(
            resp_body["account"]["addresses"][0]["address"],
            "5 rue de la justice"
        );
        assert_eq!(resp_body["account"]["addresses"][0]["postal"], "60000");
        assert_eq!(resp_body["account"]["addresses"][0]["city"], "Perpignan");

        assert_eq!(
            resp_body["account"]["services"][0]["description"],
            "Thérapie EMDR"
        );
        assert_eq!(resp_body["account"]["services"][0]["duration"], 1);
        assert_eq!(resp_body["account"]["services"][0]["unity"], "Heure");
        assert_eq!(resp_body["account"]["services"][0]["price"], 75);
        assert_eq!(resp_body["account"]["services"][0]["status"], "unchanged");

        assert_eq!(
            resp_body["account"]["services"][1]["description"],
            "Thérapie de couple"
        );
        assert_eq!(resp_body["account"]["services"][1]["duration"], 45);
        assert_eq!(resp_body["account"]["services"][1]["unity"], "Minute");
        assert_eq!(resp_body["account"]["services"][1]["price"], 60);
        assert_eq!(resp_body["account"]["services"][1]["status"], "unchanged");

        assert_eq!(resp_body["account"]["formTemplates"], json!([]));

        assert_eq!(
            resp_body["account"]["rdvTemplates"][0]["title"],
            "Consultation individuel"
        );
        assert_eq!(
            resp_body["account"]["rdvTemplates"][0]["avatarId"],
            "A0D49C-3C5337-0"
        );
        assert_eq!(resp_body["account"]["rdvTemplates"][0]["formTemplateId"], 0);
        assert_eq!(resp_body["account"]["rdvTemplates"][0]["addressId"], 1);
        assert_eq!(resp_body["account"]["rdvTemplates"][0]["serviceId"], 1);
        assert_eq!(resp_body["account"]["rdvTemplates"][0]["max"], 1);

        assert_eq!(
            resp_body["planning"]["config"]["startCurrentDay"],
            28_800_000
        );
        assert_eq!(resp_body["planning"]["config"]["endCurrentDay"], 64_800_000);
        assert_eq!(resp_body["planning"]["usersSuggestions"], json!([]));
        assert_eq!(resp_body["planning"]["weeks"], json!([]));
    }

    #[actix_web::test]
    async fn test_sending_code_no_cookie() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_to_checking_code());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_code)),
        )
        .await;

        let req_data = CheckCodeRequest {
            code: String::from("123456"),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkcode")
            .set_json(&req_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_sending_code_expired_cookie() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_to_checking_code());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() - MAX_AGE_3M;
        let expired_token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let expired_cookie = create_cookie("token", &expired_token);

        let req_data = CheckCodeRequest {
            code: String::from("123456"),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkcode")
            .set_json(&req_data)
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
                .service(web::scope("/api").service(check_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let req_data = CheckCodeRequest {
            code: String::from("123456"),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkcode")
            .set_json(&req_data)
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
                .service(web::scope("/api").service(check_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let req_data = CheckCodeRequest {
            code: String::from("123456"),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkcode")
            .set_json(&req_data)
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
                .service(web::scope("/api").service(check_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let req_data = CheckCodeRequest {
            code: String::from("123456"),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkcode")
            .set_json(&req_data)
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
    async fn test_sending_code_pro_not_found() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_pro_not_found());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let req_data = CheckCodeRequest {
            code: String::from("123456"),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkcode")
            .set_json(&req_data)
            .cookie(cookie)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn test_sending_invalid_code() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_invalid_code());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let wrong_code = CheckCodeRequest {
            code: String::from("wrong code"),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkcode")
            .set_json(&wrong_code)
            .cookie(cookie)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;

        let resp_errors: RespErrors<Value> = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp_errors.kind, "Code");
        assert_eq!(resp_errors.reason, "Invalid");

        assert_eq!(resp_errors.errors.unwrap()["tries"], 1);
    }

    #[actix_web::test]
    async fn test_sending_too_many_attempts() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_too_many_attempts());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(check_code)),
        )
        .await;

        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expires_at = Utc::now().timestamp() + MAX_AGE_3M;
        let token = CookiePayload {
            id: user_id,
            exp_at: expires_at,
        };

        let cookie = create_cookie("token", &token);

        let wrong_code = CheckCodeRequest {
            code: String::from("wrong code"),
        };

        let req = test::TestRequest::post()
            .uri("/api/checkcode")
            .set_json(&wrong_code)
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
