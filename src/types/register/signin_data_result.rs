use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SigninDataResult {
    pub email: String,
}