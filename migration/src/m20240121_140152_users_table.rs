use entity::entities::user_entity::user_model::Entity as UserEntity;
use sea_orm_migration::{prelude::*, sea_query::extension::postgres::Type, sea_orm::Schema};
use sea_orm::{EnumIter, Iterable};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
        .create_type(
            Type::create()
                .as_enum(Language::Table)
                .values(Language::iter().skip(1))
                .to_owned(),
        )
        .await?;
        let schema = Schema::new(sea_orm::DatabaseBackend::Postgres);

        manager
            .create_table(
                schema.create_table_from_entity(UserEntity),
               /*  Table::create()
                    .table(Users::Table)
                        .if_not_exists()
                        .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                        .col(ColumnDef::new(Users::Avatar).string())
                        .col(ColumnDef::new(Users::FirstName).string().not_null())
                        .col(ColumnDef::new(Users::LastName).string().not_null())
                        .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                        .col(ColumnDef::new(Users::Phone).string().not_null().unique_key())
                        .col(ColumnDef::new(Users::Terms).boolean().not_null())
                        .col(ColumnDef::new(Users::Privacy).boolean().not_null())
                        .col(ColumnDef::new(Users::TwoFa).boolean().not_null())
                        .col(
                            ColumnDef::new(Users::Language)
                                .enumeration(Language::Table, Language::iter().skip(1)),
                        )
                        .col(ColumnDef::new(Users::CreatedAt).timestamp().not_null())
                        .to_owned(), */
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Users {
    Table,
    #[iden = "id"]
    Id,
    #[iden = "av"]
    Avatar,
    #[iden = "f"]
    FirstName,
    #[iden = "l"]
    LastName,
    #[iden = "e"]
    Email,
    #[iden = "ph"]
    Phone,
    #[iden = "t"]
    Terms,
    #[iden = "pv"]
    Privacy,
    #[iden = "two_fa"]
    TwoFa,
    #[iden = "lg"]
    Language,
    #[iden = "created_at"]
    CreatedAt,
}

#[derive(Iden, EnumIter)]
pub enum Language {
    Table,
    #[iden = "fr"]
    Fr,
    #[iden = "en"]
    En,
    #[iden = "es"]
    Es,
    #[iden = "de"]
    De,
    #[iden = "it"]
    It,
}


