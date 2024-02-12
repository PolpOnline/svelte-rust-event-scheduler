use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::entity::*;

use entity::{event, round_max_users};

use crate::sea_orm::TransactionTrait;

#[derive(Debug, Clone)]
pub struct XlsxEvent {
    pub name: String,
    pub room: String,
    pub zone: String,
    pub floor: String,
    pub minimum_section: i32,
    pub round_max_users: Vec<XlsxRoundMaxUsers>,
}

#[derive(Debug, Clone)]
pub struct XlsxRoundMaxUsers {
    pub round: i32,
    pub event_id: i32,
    pub max_users: i32,
}

mod xlsx_deserialization {
    use calamine::{open_workbook, RangeDeserializerBuilder, Reader, Xlsx};

    use super::{XlsxEvent, XlsxRoundMaxUsers};

    pub(crate) fn parse_xlsx() -> Result<Vec<XlsxEvent>, calamine::Error> {
        let path = format!(
            "{}/../seed/xlsx/ATTIVITÀ FORUM DEFINITIVE.xlsx",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut workbook: Xlsx<_> = open_workbook(path)?;

        let range = workbook.worksheet_range("Foglio1")?.range((0, 0), (62, 8));

        let iter = RangeDeserializerBuilder::with_headers(&[
            "ZONA",
            "PIANO",
            "Aula",
            "n.Alunni",
            "Attività",
            "1°giorno",
            "2°giorno",
            "turno?",
        ])
        .from_range(&range)?;

        let mut events: Vec<XlsxEvent> = Vec::with_capacity(range.height());

        for (event_id, result) in iter.enumerate().map(|(i, r)| (i + 1, r)) {
            let (zone, floor, room, max_users, name, first_day, second_day, turn): (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
            ) = result?;

            let max_users = max_users.parse().unwrap();

            let turn: Option<i32> = if turn.is_empty() {
                None
            } else {
                Some(turn.parse().unwrap())
            };

            // Each day has 2 rounds.
            // The First day has round 1 and 2, the second day has round 3 and 4.
            // First day/second day is like "XX" or "X":
            // - The first means the first day has both XlsxRoundMaxUsers set to max_users
            // - The second means the second day has only the round (on a daily basis) specified by turn set to max_users
            // For example:
            // - If first_day is "XX" and second_day is "X",
            //  the first day has both rounds set to max_users
            // the second day has only the round specified by turn set to max_users
            // the other one is set to 0

            let mut round_max_users = Vec::with_capacity(4);
            let days = vec![first_day, second_day];

            for (i, day) in days.iter().enumerate() {
                if day == "XX" {
                    round_max_users.push(XlsxRoundMaxUsers {
                        round: i as i32 * 2 + 1,
                        event_id: event_id as i32,
                        max_users,
                    });
                    round_max_users.push(XlsxRoundMaxUsers {
                        round: i as i32 * 2 + 2,
                        event_id: event_id as i32,
                        max_users,
                    });
                } else if day == "X" {
                    round_max_users.push(XlsxRoundMaxUsers {
                        round: i as i32 * 2 + 1,
                        event_id: event_id as i32,
                        max_users: if turn == Some(1) { max_users } else { 0 },
                    });
                    round_max_users.push(XlsxRoundMaxUsers {
                        round: i as i32 * 2 + 2,
                        event_id: event_id as i32,
                        max_users: if turn == Some(2) { max_users } else { 0 },
                    });
                } else {
                    round_max_users.push(XlsxRoundMaxUsers {
                        round: i as i32 * 2 + 1,
                        event_id: event_id as i32,
                        max_users: 0,
                    });
                    round_max_users.push(XlsxRoundMaxUsers {
                        round: i as i32 * 2 + 2,
                        event_id: event_id as i32,
                        max_users: 0,
                    });
                }
            }

            events.push(XlsxEvent {
                name,
                room,
                zone,
                floor,
                minimum_section: 1,
                round_max_users,
            });
        }

        Ok(events)
    }
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let events = xlsx_deserialization::parse_xlsx().unwrap();

        let db = manager.get_connection();
        let transaction = db.begin().await?;

        for xlsx_event in events.clone() {
            event::ActiveModel {
                id: Default::default(),
                name: Set(xlsx_event.name),
                room: Set(xlsx_event.room),
                zone: Set(xlsx_event.zone),
                floor: Set(xlsx_event.floor),
                minimum_section: Set(xlsx_event.minimum_section),
            }
            .insert(&transaction)
            .await?;
        }

        for xlsx_max_users in events.iter().flat_map(|e| e.round_max_users.iter()) {
            round_max_users::ActiveModel {
                round: Set(xlsx_max_users.round),
                event_id: Set(xlsx_max_users.event_id),
                max_users: Set(xlsx_max_users.max_users),
            }
            .insert(&transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Delete all entries from the Event and RoundMaxUsers tables
        let db = manager.get_connection();
        let transaction = db.begin().await?;

        event::Entity::delete_many().exec(&transaction).await?;

        round_max_users::Entity::delete_many()
            .exec(&transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }
}
