use dotenv::dotenv;
use std::env;
use tracing::error;

use reqwest::header;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct AuthCodeSmsData {
    pub phone: String,
    pub first_name: String,
    pub code: String,
}

pub async fn send_auth_code_sms(data: AuthCodeSmsData) -> Result<(), ()> {
    if data.phone == "0600000001"
        || data.phone == "0600000002"
        || data.phone == "0600000003"
        || data.phone == "0600000004"
        || data.phone == "0600000005"
    {
        return Ok(());
    }

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    dotenv().ok();
    let api_key = env::var("SMS_API_KEY_SENDINBLUE").expect("Failed to get SMS_API_KEY_SENDINBLUE");

    let (_, last) = data.phone.split_at(1);
    let user_phone = String::from("+33") + &last;

    let body = json!({
        "sender": "FOCUS",
        "recipient": user_phone,
        "content": format!("Bonjour {},\nVotre code de vérification Focus est: {}.", data.first_name, data.code) ,
    });

    let res = client
        .post("https://api.brevo.com/v3/transactionalSMS/sms")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::HOST, "api.brevo.com")
        .header("api-key", api_key)
        .json(&body)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status() == 201 {
                return Ok(());
            } else {
                error!("Failed to send sms: {:?}", response.text().await.unwrap());
                return Err(());
            }
        }
        Err(err) => {
            error!("Failed to send sms: {:?}", err);
            return Err(());
        }
    }
}
