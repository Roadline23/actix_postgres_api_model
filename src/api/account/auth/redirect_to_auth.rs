use actix_web::{
    get,
    web::{Data, Json},
    HttpResponse,
};
use sea_orm::DatabaseConnection;

#[get("/<id>")]
pub async fn redirect_to_auth_rdv(db: Data<DatabaseConnection>, id: String) -> HttpResponse {
    HttpResponse::PermanentRedirect()
        .append_header(("Location", "https://www.google.com"))
        .finish()
}
