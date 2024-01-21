use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, Deserialize, Serialize)]
pub enum CrudStatus {
    #[serde(rename = "new")]
    New,
    #[serde(rename = "unchanged")]
    Unchanged,
    #[serde(rename = "updated")]
    Updated,
    #[serde(rename = "deleted")]
    Deleted,
}