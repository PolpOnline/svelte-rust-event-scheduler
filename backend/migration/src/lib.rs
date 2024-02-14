pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20240212_080458_import_xlsx_file;
mod m20240214_000121_import_csv;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240212_080458_import_xlsx_file::Migration),
            Box::new(m20240214_000121_import_csv::Migration),
        ]
    }
}
