use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use chrono::Utc;
use sea_orm::DatabaseConnection;
use serde_json::json;

use crate::{
    error::{
        errors::signin_data::{
            signin_data_check::SignInDataCheck, signin_data_errors::SignInDataErrors,
        },
        bad_request,
    },
    types::register::signin_data_result::SigninDataResult,
    utils::{
        cookie_utils::{create_cookie, CookiePayload},
        time_utils::MAX_AGE_1H_TEST,
    },
};
use service::query::user_queries::UserQuery;

#[post("/connection_test_pro")]
pub async fn connection_test_pro(
    db: Data<DatabaseConnection>,
    pro: Json<SigninDataResult>,
) -> HttpResponse {
    let email = match SignInDataCheck::new(pro.email.to_string()).validate() {
        Ok(e) => e.email,
        Err(form_error) => {
            return bad_request("Form", "Invalid", Some(form_error))
        }
    };

    let user = match UserQuery::find_user_by_email(&db, &email).await {
        Ok(p) => p,
        Err(_) => {
            return bad_request("Form", "Unknown", Some(SignInDataErrors::new("not_found")))
        }
    };

    let token = CookiePayload {
        id: user.id,
        exp_at: Utc::now().timestamp() + MAX_AGE_1H_TEST,
    };
    let session_cookie = create_cookie("SESSIONID", &token);


    let account = json!({
        "firstName": user.f,
        "lastName": user.l,
    });

    return HttpResponse::Ok().cookie(session_cookie).json(json!({
        "connection test pro": "ok",
        "account": account,
    }));
}
