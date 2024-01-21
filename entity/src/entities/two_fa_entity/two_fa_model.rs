use super::super::user_entity::user_model::Entity as UserEntity;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "two_fa")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub v_e: bool,
    pub t: i32,
    pub s: i32,
    pub c: Option<String>,
    pub up: Option<i64>,
    pub ex: i32,
    pub v_ph: bool,
    #[sea_orm(unique)]
    pub user_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::user_entity::user_model::Entity",
        from = "Column::UserId",
        to = "super::super::user_entity::user_model::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<UserEntity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
     fn new() -> Self {
        use sea_orm::Set;

        Self {
            v_e: Set(false),
            t: Set(3),
            s: Set(0),
            c: Set(None),
            up: Set(None),
            ex: Set(0),
            v_ph: Set(false),
            ..ActiveModelTrait::default()
        }
    }
}
