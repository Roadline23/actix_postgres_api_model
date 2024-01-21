use ::entity::entities::user_entity::{user_model, user_model::Entity as UserEntity};

use sea_orm::{prelude::Uuid, *};

pub struct UserMutation;

impl UserMutation {
    pub async fn create_user(
        db: &DbConn,
        form_data: user_model::ActiveModel,
    ) -> Result<user_model::Model, DbErr> {
        form_data.insert(db).await
    }

    pub async fn update_user(
        db: &DbConn,
        user: user_model::Model,
    ) -> Result<user_model::Model, DbErr> {
        user_model::ActiveModel {
            id: Set(user.id),
            av: Set(user.av),
            f: Set(user.f),
            l: Set(user.l),
            e: Set(user.e),
            ph: Set(user.ph),
            t: Set(user.t),
            pv: Set(user.pv),
            two_fa: Set(user.two_fa),
            lg: Set(user.lg),
            created_at: Set(user.created_at),
        }.update(db).await
    }

    pub async fn update_user_by_id(
        db: &DbConn,
        id: Uuid,
        form_data: user_model::Model,
    ) -> Result<user_model::Model, DbErr> {
        let user: user_model::ActiveModel = UserEntity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        user_model::ActiveModel {
            id: user.id,
            av: Set(form_data.av.to_owned()),
            f: Set(form_data.f.to_owned()),
            l: Set(form_data.l.to_owned()),
            e: Set(form_data.e.to_owned()),
            ph: Set(form_data.ph.to_owned()),
            t: Set(form_data.t.to_owned()),
            pv: Set(form_data.pv.to_owned()),
            two_fa: Set(form_data.two_fa.to_owned()),
            lg: Set(form_data.lg.to_owned()),
            created_at: Set(form_data.created_at.to_owned()),
        }
        .update(db)
        .await
    }

    pub async fn delete_user(db: &DbConn, user: user_model::Model) -> Result<DeleteResult, DbErr> {
        user.delete(db).await
    }

    pub async fn delete_user_by_id(db: &DbConn, id: Uuid) -> Result<DeleteResult, DbErr> {
        UserEntity::delete_by_id(id).exec(db).await
    }
}
