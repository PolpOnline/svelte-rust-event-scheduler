use crate::sea_orm::TransactionTrait;
use entity::user;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::entity::*;
use serde::Deserialize;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Deserialize)]
pub struct User {
    pub name: String,
    pub email: String,
    pub section: i32,
}

mod csv_deserialization {
    use csv::ReaderBuilder;

    use super::User;

    pub(crate) fn parse_csv() -> Result<Vec<User>, csv::Error> {
        let path = format!(
            "{}/../seed/csv/ATTIVITÀ FORUM DEFINITIVE.csv",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?;

        let mut users: Vec<User> = Vec::new();

        for result in reader.deserialize() {
            let user: User = result?;
            users.push(user);
        }

        Ok(users)
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let users = csv_deserialization::parse_csv()
            .unwrap()
            .iter()
            .map(|user| user::ActiveModel {
                name: Set(Some(user.name.clone())),
                email: Set(user.email.clone()),
                section: Set(user.section),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let transaction = db.begin().await?;

        user::Entity::insert_many(users).exec(&transaction).await?;

        transaction.commit().await?;

        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let transaction = db.begin().await?;

        user::Entity::delete_many().exec(&transaction).await?;

        transaction.commit().await?;

        Ok(())
    }
}
