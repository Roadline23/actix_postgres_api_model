pub use sea_orm_migration::prelude::*;

mod m20240121_140152_users_table;
mod m20240121_140719_two_fa_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240121_140152_users_table::Migration),
            Box::new(m20240121_140719_two_fa_table::Migration),
        ]
    }
}
