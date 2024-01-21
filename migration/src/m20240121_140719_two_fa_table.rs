use sea_orm_migration::{prelude::*, sea_orm::Schema};

use entity::entities::two_fa_entity::two_fa_model::Entity as TwoFaEntity;


#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(sea_orm::DatabaseBackend::Postgres);

        manager
            .create_table(
                schema.create_table_from_entity(TwoFaEntity)
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TwoFa::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TwoFa {
    Table,
}
