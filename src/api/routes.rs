use actix_web::web;

use super::account::{
    auth::{check_code_api::check_code, check_email_api::check_email, send_code_api::send_code},
    delete::delete_user::delete_user,
    register::{connection_test_pro_api::connection_test_pro, signin_api::sign_in_pro, signup_api::sign_up_pro},
};

pub fn init_auth_pro_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(sign_up_pro);
    cfg.service(sign_in_pro);
    cfg.service(connection_test_pro);
    cfg.service(check_email);
    cfg.service(send_code);
    cfg.service(check_code);
}
