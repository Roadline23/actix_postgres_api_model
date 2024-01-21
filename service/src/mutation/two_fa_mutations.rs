use ::entity::entities::two_fa_entity::{two_fa_model, two_fa_model::Entity as TwoFaEntity};
use sea_orm::*;

pub struct TwoFaMutation;

impl TwoFaMutation {
    pub async fn create_two_fa(
        db: &DbConn,
        form_data: two_fa_model::ActiveModel,
    ) -> Result<two_fa_model::Model, DbErr> {
        form_data.insert(db).await
    }

    pub async fn update_two_fa(
        db: &DbConn,
        form_data: two_fa_model::ActiveModel,
    ) -> Result<two_fa_model::Model, DbErr> {
        form_data.update(db).await
    }

    pub async fn update_two_fa_by_id(
        db: &DbConn,
        id: i32,
        form_data: two_fa_model::Model,
    ) -> Result<two_fa_model::Model, DbErr> {
        two_fa_model::ActiveModel {
            id: Set(id),
            v_e: Set(form_data.v_e),
            t: Set(form_data.t),
            s: Set(form_data.s),
            c: Set(form_data.c),
            up: Set(form_data.up),
            ex: Set(form_data.ex),
            v_ph: Set(form_data.v_ph),
            user_id: Set(form_data.user_id),
        }
        .update(db)
        .await
    }

    pub async fn delete_two_fa(
        db: &DbConn,
        two_fa: two_fa_model::Model,
    ) -> Result<DeleteResult, DbErr> {
        two_fa.delete(db).await
    }
}
