use std::collections::HashSet;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::utils::{
    string::format_into_string_utils::format_validation_error,
    validate_utils::{has_errors, must_accept, required},
};
use lazy_static::lazy_static;
use validator::Validate;

use super::signup_data_errors::SignUpDataErrors;

lazy_static! {
    static ref ONLY_ALPHABETIC: Regex = Regex::new(r"^[a-zA-Z]+$").unwrap();
    static ref PHONE_REGEX: Regex = Regex::new(r"^(\+33|0)(6|7|9)(\d{2}){4}$").unwrap();
    static ref SIREN_REGEX: Regex = Regex::new(r"^\d{9}$").unwrap();
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SignUpDataCheck {
    #[validate(
        custom(function = "required"),
        length(min = 2, code = "not_a_first_name"),
        length(max = 40, code = "only_the_first"),
        regex(path = "ONLY_ALPHABETIC", code = "not_a_first_name")
    )]
    pub firstName: String,
    #[validate(
        custom = "required",
        length(min = 2, code = "not_a_last_name"),
        length(max = 40, code = "only_the_first"),
        regex(path = "ONLY_ALPHABETIC", code = "not_a_last_name")
    )]
    pub lastName: String,
    #[validate(custom = "required", email(code = "invalid"))]
    pub email: String,
    #[validate(custom = "required", regex(path = "PHONE_REGEX", code = "invalid"))]
    pub phone: String,
    #[validate(custom = "required", regex(path = "SIREN_REGEX", code = "not_a_siren"))]
    pub siren: String,
    #[validate(custom = "must_accept")]
    pub terms: bool,
    #[validate(custom = "must_accept")]
    pub privacy: bool,
}

impl SignUpDataCheck {
    pub fn validate(&self) -> Option<SignUpDataErrors> {
        match validator::Validate::validate(self) {
            Ok(_) => None,
            Err(err) => {
                let mut signup_data_errors = SignUpDataErrors::new();
                let validation_errors_json = serde_json::json!(err);
                for (key, value) in validation_errors_json.as_object().unwrap() {
                    match key.as_str() {
                        "firstName" => {
                            signup_data_errors.firstName = format_validation_error(&value);
                        }
                        "lastName" => {
                            signup_data_errors.lastName = format_validation_error(&value);
                        }
                        "email" => {
                            signup_data_errors.email = format_validation_error(&value);
                        }
                        "phone" => {
                            signup_data_errors.phone = format_validation_error(&value);
                        }
                        "siren" => {
                            signup_data_errors.siren = format_validation_error(&value);
                        }
                        "terms" => {
                            signup_data_errors.terms = format_validation_error(&value);
                        }
                        "privacy" => {
                            signup_data_errors.privacy = format_validation_error(&value);
                        }
                        _ => (),
                    }
                }
                let signup_data_vec = vec![
                    &signup_data_errors.firstName,
                    &signup_data_errors.lastName,
                    &signup_data_errors.email,
                    &signup_data_errors.phone,
                    &signup_data_errors.siren,
                    &signup_data_errors.terms,
                    &signup_data_errors.privacy,
                ];

                match has_errors(signup_data_vec) {
                    false => None,
                    true => Some(signup_data_errors),
                }
            }
        }
    }
}
