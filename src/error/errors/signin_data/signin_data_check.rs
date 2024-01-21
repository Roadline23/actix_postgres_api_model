use serde::{Deserialize, Serialize};

use crate::utils::{validate_utils::required, string::format_into_string_utils::format_validation_error};
use validator::Validate;

use super::signin_data_errors::SignInDataErrors;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SignInDataCheck {
    #[validate(custom = "required", email(code = "invalid"))]
    pub email: String,
}

impl SignInDataCheck {
    pub fn new(email: String) -> Self {
        Self { email }
    }

    pub fn validate(self) -> Result<SignInDataCheck, SignInDataErrors> {
        match validator::Validate::validate(&self) {
            Ok(_) => Ok(self),
            Err(err) => {
                let mut signup_data_errors = SignInDataErrors::new("");
                let validation_errors_json = serde_json::json!(err);
                for (key, value) in validation_errors_json.as_object().unwrap() {
                    match key.as_str() {
                        "email" => {
                            signup_data_errors.email = format_validation_error(&value);
                        }
                        _ => (),
                    }
                }
                Err(signup_data_errors)
            }
        }
    }
}
