use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CheckEmailDataRequest {
    pub token: Option<String>,
}