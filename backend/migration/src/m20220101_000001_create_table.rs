use sea_orm_migration::prelude::*;

/// Prisma code we are migrating from:
///
/// ```prisma
/// model Event {
/// id             Int         @id @default(autoincrement())
/// name           String
/// description    String
/// EventUser      EventUser[]
/// minimumSection Int         @default(0)
///
/// RoundMaxUsers RoundMaxUsers[]
/// }
///
/// model User {
/// id              Int         @id @default(autoincrement())
/// name            String?
/// email           String      @unique
/// EventUser       EventUser[]
/// interactiveDone Boolean     @default(false)
/// section         Int         @default(1)
/// class           String?
/// admin           Boolean     @default(false)
///
/// @@index([email])
/// }
///
/// model EventUser {
/// event    Event     @relation(references: [id], fields: [eventId])
/// user     User      @relation(references: [id], fields: [userId])
/// joinedAt DateTime?
/// round    Int
///
/// eventId Int
/// userId  Int
///
/// @@id([userId, round])
/// }
///
/// model RoundMaxUsers {
/// round   Int
/// eventId Int
///
/// maxUsers Int
///
/// event Event @relation(references: [id], fields: [eventId])
///
/// @@id([round, eventId])
/// }
/// ```
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the Event table
        manager
            .create_table(
                Table::create()
                    .table(Event::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Event::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Event::Name).string().not_null())
                    .col(ColumnDef::new(Event::Description).string().not_null())
                    .col(ColumnDef::new(Event::Room).string().not_null())
                    .col(ColumnDef::new(Event::Zone).string().not_null())
                    .col(ColumnDef::new(Event::Floor).string().not_null())
                    .col(
                        ColumnDef::new(Event::MinimumSection)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the User table
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User::Name).string())
                    .col(ColumnDef::new(User::Email).string().not_null().unique_key())
                    .col(
                        ColumnDef::new(User::InteractiveDone)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(User::Section)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(User::Class).string())
                    .col(
                        ColumnDef::new(User::Admin)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the EventUser table
        manager
            .create_table(
                Table::create()
                    .table(EventUser::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(EventUser::EventId).integer().not_null())
                    .col(ColumnDef::new(EventUser::UserId).integer().not_null())
                    .col(ColumnDef::new(EventUser::JoinedAt).date_time())
                    .col(ColumnDef::new(EventUser::Round).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_user_event_id")
                            .from_tbl(EventUser::Table)
                            .from_col(EventUser::EventId)
                            .to_tbl(Event::Table)
                            .to_col(Event::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_user_user_id")
                            .from_tbl(EventUser::Table)
                            .from_col(EventUser::UserId)
                            .to_tbl(User::Table)
                            .to_col(User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_event_user")
                            .col(EventUser::UserId)
                            .col(EventUser::Round),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the RoundMaxUsers table
        manager
            .create_table(
                Table::create()
                    .table(RoundMaxUsers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(RoundMaxUsers::Round).integer().not_null())
                    .col(ColumnDef::new(RoundMaxUsers::EventId).integer().not_null())
                    .col(ColumnDef::new(RoundMaxUsers::MaxUsers).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_round_max_users_event_id")
                            .from_tbl(RoundMaxUsers::Table)
                            .from_col(RoundMaxUsers::EventId)
                            .to_tbl(Event::Table)
                            .to_col(Event::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_round_max_users")
                            .col(RoundMaxUsers::Round)
                            .col(RoundMaxUsers::EventId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the RoundMaxUsers table
        manager
            .drop_table(Table::drop().table(RoundMaxUsers::Table).to_owned())
            .await?;

        // Drop the EventUser table
        manager
            .drop_table(Table::drop().table(EventUser::Table).to_owned())
            .await?;

        // Drop the User table
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;

        // Drop the Event table
        manager
            .drop_table(Table::drop().table(Event::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Event {
    Table,
    Id,
    Name,
    Description,
    Room,
    Zone,
    Floor,
    MinimumSection,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Name,
    Email,
    InteractiveDone,
    Section,
    Class,
    Admin,
}

#[derive(DeriveIden)]
enum EventUser {
    Table,
    EventId,
    UserId,
    JoinedAt,
    Round,
}

#[derive(DeriveIden)]
enum RoundMaxUsers {
    Table,
    Round,
    EventId,
    MaxUsers,
}
