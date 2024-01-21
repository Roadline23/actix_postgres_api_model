use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RdvConfirmWithFormEmailData {
    pub email_to: String,
    pub first_name: String,
    pub date: String,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct RdvConfirmEmailData {
    pub email_to: String,
    pub first_name: String,
    pub date: String,
    pub times: String,
    pub token: String,
}