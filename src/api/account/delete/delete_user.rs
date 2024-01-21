use crate::{
    error::{internal_server_error, ok_response},
    middlewares::app_state::AppState,
};
use actix_web::{delete, web::Data, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use service::mutation::user_mutations::UserMutation;
use tracing::error;

#[delete("/delete_account")]
pub async fn delete_user(
    req: HttpRequest,
    db: Data<DatabaseConnection>,
    user: Data<AppState>,
) -> HttpResponse {
    match UserMutation::delete_user_by_id(&db, user.id.lock().unwrap().unwrap()).await {
        Ok(_) => ok_response::<String>(None),
        Err(err) => {
            error!("Cannot delete user: {}", err);
            return internal_server_error::<String>("User", "DeletionFailure", None);
        }
    }
}
