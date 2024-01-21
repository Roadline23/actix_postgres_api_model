use super::super::two_fa_entity::two_fa_model::Entity as TwoFaEntity;
use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, FromQueryResult};
use serde::{Deserialize, Serialize};

#[derive(FromQueryResult, Deserialize, Serialize, Debug, Clone)]
pub struct UsersToRdv {
    #[serde(skip_deserializing)]
    pub id: Uuid,
    #[serde(rename = "uuidKey")]
    pub uuid_key: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub status: String,
}

#[derive(FromQueryResult, Deserialize, Serialize, Debug, Clone)]
pub struct PartialUsersToRdv {
    #[serde(skip_deserializing)]
    pub id: Uuid,
    pub f: String,
    pub l: String,
    pub e: String,
    pub ph: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub id: Uuid,
    pub av: Option<String>,
    pub f: String,
    pub l: String,
    #[sea_orm(unique)]
    pub e: String,
    #[sea_orm(unique)]
    pub ph: String,
    pub t: bool,
    pub pv: bool,
    pub two_fa: bool,
    pub lg: Language,
    pub created_at: DateTime<Utc>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, EnumIter, DeriveActiveEnum, Deserialize, Serialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "language")]
pub enum Language {
    #[default]
    #[sea_orm(string_value = "fr")]
    Fr,
    #[sea_orm(string_value = "en")]
    En,
    #[sea_orm(string_value = "es")]
    Es,
    #[sea_orm(string_value = "de")]
    De,
    #[sea_orm(string_value = "it")]
    It,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::super::two_fa_entity::two_fa_model::Entity")]
    TwoFa,
}

impl Related<TwoFaEntity> for Entity {
    fn to() -> RelationDef {
        Relation::TwoFa.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        use sea_orm::Set;

        Self {
            id: Set(Uuid::new_v4()),
            av: Set(None),
            two_fa: Set(true),
            lg: Set(Language::Fr),
            created_at: Set(Utc::now()),
            ..ActiveModelTrait::default()
        }
    }
}
