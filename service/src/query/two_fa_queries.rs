use ::entity::entities::two_fa_entity::{two_fa_model, two_fa_model::Entity as TwoFaEntity};
use ::entity::entities::user_entity::user_model::{Entity as UserEntity, self};
use sea_orm::*;
use uuid::Uuid; 

pub struct TwoFaQuery;

impl TwoFaQuery {
    pub async fn find_two_fa_by_id(db: &DbConn, id: i32) -> Result<Option<two_fa_model::Model>, DbErr> {
        TwoFaEntity::find_by_id(id).one(db).await
    }
}