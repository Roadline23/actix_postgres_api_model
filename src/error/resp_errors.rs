use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RespErrors<T> {
    pub kind: String,
    pub reason: String,
    pub errors: Option<T>,
}

impl<T> RespErrors<T> {
    pub fn new(kind: &str, reason: &str, errors: Option<T>) -> Self {
        RespErrors {
            kind: kind.to_string(),
            reason: reason.to_string(),
            errors,
        }
    }
}
