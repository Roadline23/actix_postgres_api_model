use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use chrono::Utc;
use sea_orm::DatabaseConnection;
use serde_json::json;

use crate::{
    emails::two_factor_auth_email::{send_two_factor_auth_email, TwoFactorAuthEmailData},
    error::{
        bad_request,
        errors::signin_data::{
            signin_data_check::SignInDataCheck, signin_data_errors::SignInDataErrors,
        },
    },
    types::register::signin_data_result::SigninDataResult,
    utils::{
        jwt_utils::create_token, time_utils::MAX_AGE_3M, two_factors_auth_utils::TwoFactorsAuth,
    },
};
use service::query::user_queries::UserQuery;

#[post("/signin")]
pub async fn sign_in_pro(
    db: Data<DatabaseConnection>,
    pro: Json<SigninDataResult>,
) -> HttpResponse {
    let email = match SignInDataCheck::new(pro.email.to_string()).validate() {
        Ok(e) => e.email,
        Err(invalid_email_error) => {
            return bad_request("Form", "Invalid", Some(invalid_email_error));
        }
    };

    let user = match UserQuery::find_user_by_email(&db, &email).await {
        Ok(p) => p,
        Err(_) => return bad_request("Form", "Unknown", Some(SignInDataErrors::new("not_found"))),
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
        let data_to_email = TwoFactorAuthEmailData {
            first_name: user.f,
            email_to: user.e,
            token: create_token(&user.id, Utc::now().timestamp() + MAX_AGE_3M),
        };

        match send_two_factor_auth_email(data_to_email).await {
            Ok(()) => HttpResponse::Ok().finish(),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::resp_errors::RespErrors;
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

    use super::{sign_in_pro, SignInDataErrors, SigninDataResult};

    fn mock_db_user_not_found() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors(vec![DbErr::RecordNotFound("not found".to_string())])
            .into_connection()
    }

    fn mock_db_with_verified_account() -> DatabaseConnection {
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
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                ..Default::default()
            }]])
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
                up: Some((Utc::now().timestamp_millis() as i64) + 300_000), // 5 minutes left
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                ..Default::default()
            }]])
            .into_connection()
    }
    #[actix_web::test]
    async fn test_sign_in_pro_success() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_with_verified_account());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(sign_in_pro)),
        )
        .await;

        let signin_data = SigninDataResult {
            email: String::from("test.pro.1@gmail.com"),
        };

        let req = test::TestRequest::post()
            .uri("/api/signin")
            .set_json(&signin_data)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_sign_in_pro_account_blocked() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_with_blocked_account());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(sign_in_pro)),
        )
        .await;

        let signin_data = SigninDataResult {
            email: String::from("test.pro.1@gmail.com"),
        };

        let req = test::TestRequest::post()
            .uri("/api/signin")
            .set_json(&signin_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;

        let resp_errors: RespErrors<Value> = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp_errors.kind, "Auth");
        assert_eq!(resp_errors.reason, "TimeLeft");
    }

    #[actix_web::test]
    async fn test_sign_in_pro_account_invalid_email() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_with_verified_account());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(sign_in_pro)),
        )
        .await;

        let signin_data = SigninDataResult {
            email: String::from("incorrect_email@"),
        };

        let req = test::TestRequest::post()
            .uri("/api/signin")
            .set_json(&signin_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;

        let resp_errors: RespErrors<SignInDataErrors> = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp_errors.kind, "Form");
        assert_eq!(resp_errors.reason, "Invalid");
        assert_eq!(resp_errors.errors.unwrap().email, String::from("invalid"));
    }

    #[actix_web::test]
    async fn test_sign_in_pro_unknown_account() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_user_not_found());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(sign_in_pro)),
        )
        .await;

        let signin_data = SigninDataResult {
            email: String::from("team.maucalli@gmail.com"),
        };

        let req = test::TestRequest::post()
            .uri("/api/signin")
            .set_json(&signin_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;
        let resp_errors: RespErrors<SignInDataErrors> = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp_errors.kind, "Form");
        assert_eq!(resp_errors.reason, "Unknown");
        assert_eq!(resp_errors.errors.unwrap().email, String::from("not_found"));
    }
}
