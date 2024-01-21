use crate::{
    emails::two_factor_auth_email::{send_two_factor_auth_email, TwoFactorAuthEmailData},
    error::{
        bad_request,
        errors::{
            signup_data::{
                signup_data_check::SignUpDataCheck, signup_data_errors::SignUpDataErrors,
            },
        },
        internal_server_error,
    },
    types::register::signup_data_result::SignUpDataResult,
    utils::{
        jwt_utils::create_token,
        time_utils::MAX_AGE_3M,
    },
};
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};

use chrono::Utc;
use entity::entities::{
    two_fa_entity::two_fa_model,
    user_entity::user_model,
};
use sea_orm::{prelude::*, ActiveValue::Set, DatabaseConnection};

use service::mutation::two_fa_mutations::TwoFaMutation;
use service::mutation::user_mutations::UserMutation;
use tracing::error;

#[post("/signup")]
pub async fn sign_up_pro(
    db: Data<DatabaseConnection>,
    new_pro: Json<SignUpDataResult>,
) -> HttpResponse {
    let signup_data_check = SignUpDataCheck {
        firstName: new_pro.firstName.to_string(),
        lastName: new_pro.lastName.to_string(),
        email: new_pro.email.to_string(),
        phone: new_pro.phone.to_string(),
        siren: new_pro.siren.to_string(),
        terms: new_pro.terms.to_owned(),
        privacy: new_pro.privacy.to_owned(),
    };

    match signup_data_check.validate() {
        None => (),
        Some(form_errors) => {
            return bad_request("Form", "Invalid", Some(form_errors));
        }
    }

    let mut user = user_model::ActiveModel::new();
    user.f = Set(signup_data_check.firstName.to_owned());
    user.l = Set(signup_data_check.lastName.to_owned());
    user.e = Set(signup_data_check.email.to_owned());
    user.ph = Set(signup_data_check.phone.to_owned());
    user.t = Set(signup_data_check.terms.to_owned());
    user.pv = Set(signup_data_check.privacy.to_owned());

    let created_user = match UserMutation::create_user(&db, user).await {
        Ok(u) => u,
        Err(err) => match err {
            DbErr::Query(err) => {
                let error = err.to_string();
                if error.contains("duplicate key") {
                    let mut sign_up_data_errors = SignUpDataErrors::new();
                    if error.contains("users_e_key") {
                        sign_up_data_errors.email = String::from("already_exists");
                    } else if error.contains("users_ph_key") {
                        sign_up_data_errors.phone = String::from("already_exists");
                    }
                    return bad_request("Form", "Invalid", Some(sign_up_data_errors));
                } else {
                    return HttpResponse::InternalServerError().finish();
                }
            }
            _ => {
                return HttpResponse::InternalServerError().finish();
            }
        },
    };

    let mut two_fa = two_fa_model::ActiveModel::new();
    two_fa.user_id = Set(created_user.id);
    let new_two_fa = TwoFaMutation::create_two_fa(&db, two_fa).await.unwrap();

    let data_to_email = TwoFactorAuthEmailData {
        first_name: created_user.f,
        email_to: created_user.e,
        token: create_token(&created_user.id, Utc::now().timestamp() + MAX_AGE_3M),
    };

    match send_two_factor_auth_email(data_to_email).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(err) => {
            error!("SIGNUP: Email not sent, details: {:?}", err);
            return internal_server_error::<String>("Email", "NotSent", None);
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{test, web::{self, Data}, App,};
    use entity::entities::{
        two_fa_entity::two_fa_model,
        user_entity::user_model,
    };
    use reqwest::StatusCode;
    use sea_orm::{
        DatabaseBackend, DatabaseConnection, DbErr, MockDatabase, MockExecResult, RuntimeErr,
    };
    use uuid::Uuid;
    use crate::error::resp_errors::RespErrors;

    use super::{sign_up_pro, SignUpDataResult, SignUpDataErrors};

    fn mock_db_with_created_account() -> DatabaseConnection {
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

    fn mock_db_duplicated_user_email() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors(vec![DbErr::Query(RuntimeErr::Internal(
                "duplicate key: users_e_key".to_string(),
            ))])
            .into_connection()
    }

    fn mock_db_duplicated_user_phone() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors(vec![DbErr::Query(RuntimeErr::Internal(
                "duplicate key: users_ph_key".to_string(),
            ))])
            .into_connection()
    }

    #[actix_web::test]
    async fn test_sign_up_pro_success() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_with_created_account());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(sign_up_pro)),
        )
        .await;

        let payload = SignUpDataResult {
            firstName: "Rob".to_owned(),
            lastName: "Doe".to_owned(),
            email: "test.pro.1@gmail.com".to_owned(),
            phone: "0600000001".to_owned(),
            siren: "883116000".to_owned(),
            terms: true,
            privacy: true,
            denomination: Some("Rob Company".to_owned()),
            sameIdentity: true,
            checkSiren: true,
            address: Some("5 rue de la justice".to_owned()),
            postal: Some("60000".to_owned()),
            city: Some("Perpignan".to_owned()),
        };

        let req = test::TestRequest::post()
            .uri("/api/signup")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_sign_up_pro_invalid_form() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_with_created_account());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(sign_up_pro)),
        )
        .await;

        let payload = SignUpDataResult {
            firstName: "".to_owned(),
            lastName: "l".to_owned(),
            email: "invalid.email@".to_owned(),
            phone: "0102030405".to_owned(),
            siren: "8831002003".to_owned(),
            terms: false,
            privacy: false,
            denomination: None,
            sameIdentity: false,
            checkSiren: false,
            address: None,
            postal: None,
            city: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/signup")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;

        let resp_errors: RespErrors<SignUpDataErrors> = serde_json::from_slice(&body).unwrap();
        let errors = resp_errors.errors.unwrap();

        assert_eq!(resp_errors.kind, "Form");
        assert_eq!(resp_errors.reason, "Invalid");

        let required_errror = String::from("required");
        let not_a_last_name_error = String::from("not_a_last_name");
        let invalid_error = String::from("invalid");
        let must_accept_error = String::from("must_accept");
        let not_a_siren = String::from("not_a_siren");

        assert_eq!(&errors.firstName, &required_errror);
        assert_eq!(&errors.lastName, &not_a_last_name_error);
        assert_eq!(&errors.email, &invalid_error);
        assert_eq!(&errors.phone, &invalid_error);
        assert_eq!(&errors.siren, &not_a_siren);
        assert_eq!(&errors.terms, &must_accept_error);
        assert_eq!(&errors.privacy, &must_accept_error);
    }

    #[actix_web::test]
    async fn test_sign_up_pro_duplicated_user_email() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_duplicated_user_email());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(sign_up_pro)),
        )
        .await;

        let payload = SignUpDataResult {
            firstName: "Rob".to_owned(),
            lastName: "Doe".to_owned(),
            email: "test.pro.1@gmail.com".to_owned(),
            phone: "0600000001".to_owned(),
            siren: "883116000".to_owned(),
            terms: true,
            privacy: true,
            denomination: Some("Rob Company".to_owned()),
            sameIdentity: true,
            checkSiren: true,
            address: Some("5 rue de la justice".to_owned()),
            postal: Some("60000".to_owned()),
            city: Some("Perpignan".to_owned()),
        };

        let req = test::TestRequest::post()
            .uri("/api/signup")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;

        let resp_errors: RespErrors<SignUpDataErrors> = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp_errors.kind, "Form");
        assert_eq!(resp_errors.reason, "Invalid");

        let errors = resp_errors.errors.unwrap();

        assert_eq!(errors.phone, String::from(""));
        assert_eq!(errors.email, String::from("already_exists"));
    }

    #[actix_web::test]
    async fn test_sign_up_pro_duplicated_user_phone() {
        let db_data: Data<DatabaseConnection> = Data::new(mock_db_duplicated_user_phone());

        let app = test::init_service(
            App::new()
                .app_data(db_data)
                .service(web::scope("/api").service(sign_up_pro)),
        )
        .await;

        let payload = SignUpDataResult {
            firstName: "Rob".to_owned(),
            lastName: "Doe".to_owned(),
            email: "test.pro.1@gmail.com".to_owned(),
            phone: "0600000001".to_owned(),
            siren: "883116000".to_owned(),
            terms: true,
            privacy: true,
            denomination: Some("Rob Company".to_owned()),
            sameIdentity: true,
            checkSiren: true,
            address: Some("5 rue de la justice".to_owned()),
            postal: Some("60000".to_owned()),
            city: Some("Perpignan".to_owned()),
        };

        let req = test::TestRequest::post()
            .uri("/api/signup")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;

        let resp_errors: RespErrors<SignUpDataErrors> = serde_json::from_slice(&body).unwrap();

        assert_eq!(resp_errors.kind, "Form");
        assert_eq!(resp_errors.reason, "Invalid");

        let errors = resp_errors.errors.unwrap();

        assert_eq!(errors.email, String::from(""));
        assert_eq!(errors.phone, String::from("already_exists"));
    }

}
