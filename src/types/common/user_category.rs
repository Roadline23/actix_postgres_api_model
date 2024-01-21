use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, Deserialize, Serialize)]
pub enum UserCategory {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "temporary")]
    Temporary,
    #[serde(rename = "not_found")]
    NotFound,
}
