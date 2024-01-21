use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CheckCodeRequest {
    pub code: String,
}