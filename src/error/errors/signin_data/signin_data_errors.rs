use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct SignInDataErrors {
    pub email: String,
}

impl SignInDataErrors {
    pub fn new(error: &str) -> Self {
        SignInDataErrors {
            email: error.to_string(),
        }
    }
}
