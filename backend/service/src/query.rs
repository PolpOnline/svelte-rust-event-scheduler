use ::entity::{event, event_user, prelude::*, user};
use sea_orm::*;

pub struct Query;

#[derive(Debug)]
pub struct EventUserStatus {
    pub id: i32,
    pub name: Option<String>,
    pub email: String,
    pub section: i32,
    pub class: Option<String>,
    pub joined_at: Option<chrono::NaiveDateTime>,
    pub left_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug)]
pub struct EventCounterStatus {
    pub event_id: i32,
    pub count: u64,
}

impl Query {
    pub async fn get_all_events(db: &DbConn) -> Result<Vec<event::Model>, DbErr> {
        let events = Event::find()
            .order_by(event::Column::Id, Order::Asc)
            .all(db)
            .await?;

        Ok(events)
    }

    pub async fn get_event_user_count_by_id(db: &DbConn, event_id: i32) -> Result<u64, DbErr> {
        let count = EventUser::find()
            .filter(event_user::Column::EventId.eq(event_id))
            .count(db)
            .await?;

        Ok(count)
    }

    pub async fn get_events_user_count_by_ids(
        db: &DbConn,
        event_ids: Vec<i32>,
    ) -> Result<Vec<EventCounterStatus>, DbErr> {
        let mut counts = Vec::with_capacity(event_ids.len());

        for event_id in event_ids {
            let count = EventUser::find()
                .filter(event_user::Column::EventId.eq(event_id))
                .count(db)
                .await?;

            counts.push(EventCounterStatus { event_id, count });
        }

        Ok(counts)
    }

    pub async fn event_users_status(
        db: &DbConn,
        event_id: i32,
        round: i32,
    ) -> Result<Vec<EventUserStatus>, DbErr> {
        let event_users: Vec<event_user::Model> = event_user::Entity::find()
            .filter(event_user::Column::EventId.eq(event_id))
            .filter(event_user::Column::Round.eq(round))
            .all(db)
            .await?;

        let mut event_users_status = Vec::with_capacity(event_users.len());

        for event_user in event_users {
            let user = user::Entity::find_by_id(event_user.user_id)
                .one(db)
                .await?
                .unwrap();

            let status = EventUserStatus {
                id: user.id,
                name: user.name,
                email: user.email,
                section: user.section,
                class: user.class,
                joined_at: event_user.joined_at,
                left_at: event_user.left_at,
            };

            event_users_status.push(status);
        }

        Ok(event_users_status)
    }
}
