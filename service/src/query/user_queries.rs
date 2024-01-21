use ::entity::entities::{
    two_fa_entity::two_fa_model::{Entity as TwoFaEntity, Model as TwoFaModel},
    user_entity::{user_model, user_model::Entity as UserEntity},
};
use sea_orm::*;
use tracing::{error, warn};
use uuid::Uuid;

pub struct UserQuery;

impl UserQuery {
    pub async fn find_user_by_id(db: &DbConn, id: Uuid) -> Result<user_model::Model, DbErr> {
        match UserEntity::find_by_id(id).one(db).await {
            Ok(user) => match user {
                Some(user) => Ok(user),
                None => {
                    warn!("Cannot find user by id: {}", id);
                    Err(DbErr::RecordNotFound(format!("id not found: {}", id)))
                }
            },
            Err(err) => {
                error!("Cannot find user by id: {}", err);
                Err(err)
            }
        }
    }
    
    pub async fn find_user_by_email(
        db: &DbConn,
        email: &String,
    ) -> Result<user_model::Model, ()> {
        match UserEntity::find()
            .filter(user_model::Column::E.contains(email))
            .one(db)
            .await
        {
            Ok(user) => match user {
                Some(user) => Ok(user),
                None => {
                    warn!("Cannot find user by email: {}", email);
                   Err(())
                }
            },
            Err(err) => {
                error!("Cannot find user by email: {}", err);
                Err(())
            }
        }
    }

    pub async fn find_related_two_fa(
        db: &DbConn,
        user: &user_model::Model,
    ) -> Result<TwoFaModel, DbErr> {
        match user.find_related(TwoFaEntity).one(db).await {
            Ok(two_fa) => match two_fa {
                Some(t) => Ok(t),
                None => {
                    warn!("Cannot find related two_fa, req by this user: {:#?}", user);
                    Err(DbErr::RecordNotFound(format!(
                        "two_fa not found, req by this user: {:#?}",
                        user
                    )))
                }
            },
            Err(err) => {
                error!(
                    "Cannot find related two_fa err: {} req by this user: {:#?}",
                    err, user
                );
                Err(err)
            }
        }
    }
}
