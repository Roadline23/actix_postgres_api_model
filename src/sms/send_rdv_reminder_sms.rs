use dotenv::dotenv;
use std::env;
use tracing::error;

use reqwest::header;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct RdvReminderSmsData {
    pub date: String,
    pub timeslot: String,
    pub pro_full_name: String,
    pub user_phone: String,
    pub shorten_url: String,
}

pub async fn send_rdv_reminder_sms(data: RdvReminderSmsData) -> () {
    if data.user_phone == "0600000001"
        || data.user_phone == "0600000002"
        || data.user_phone == "0600000003"
        || data.user_phone == "0600000004"
        || data.user_phone == "0600000005"
    {
        return ();
    }

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    dotenv().ok();
    let api_key = env::var("SMS_API_KEY_SENDINBLUE").expect("Failed to get SMS_API_KEY_SENDINBLUE");

    let (_, last) = data.user_phone.split_at(1);
    let user_phone = String::from("+33") + &last;

    let body = json!({
        "sender": "FOCUS",
        "recipient": user_phone,
        "content": format!("Bonjour\nRDV {} à {}\n{}\nInfos et annulation: {}.", data.date, data.timeslot, data.pro_full_name, data.shorten_url) ,
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
            if response.status() != 201 {
                error!("Failed to send sms: {:?}", response.text().await.unwrap());
            }
        }
        Err(err) => {
            error!("Failed to send sms: {:?}", err);
        }
    }
}
