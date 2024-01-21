use dotenv::dotenv;
use std::env;
use tracing::error;
use reqwest::header;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct TwoFactorAuthEmailData {
    pub email_to: String,
    pub first_name: String,
    pub token: String,
}

pub async fn send_two_factor_auth_email(data: TwoFactorAuthEmailData) -> Result<(), ()> {
    if data.email_to.contains("test") {
        return Ok(());
    }
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    dotenv().ok();
    let saas_root = env::var("SAAS_ROOT").expect("Error loading env var");
    let api_key = env::var("EMAIL_API_KEY_SENDINBLUE").expect("Error loading env var");

    let body = json!({
        "to": [{"email": data.email_to}],
        "templateId": 6,
        "params": {"firstName": data.first_name, "url": format!("{}/check/?t={}", saas_root, data.token) },
        "headers": {"X-Mailin-custom": "custom_header_1:custom_value_1|custom_header_2:custom_value_2|custom_header_3:custom_value_3", "charset": "iso-8859-1"}
    });

    let res = client
        .post("https://api.brevo.com/v3/smtp/email")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .header("api-key", api_key)
        .json(&body)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status() == 201 {
                Ok(())
            } else {
                error!("Auth email, details: {:?}", response.text().await.unwrap());
                return Err(());
            }
        }
        Err(e) => {
            error!("Auth email, details: {:?}", e);
            return Err(());
        }
    }
}
