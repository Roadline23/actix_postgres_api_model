use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    pub id: Mutex<Option<Uuid>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            id: Mutex::new(None),
        }
    }
}
