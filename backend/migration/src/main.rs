use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    color_eyre::install().unwrap();

    cli::run_cli(migration::Migrator).await;
}
