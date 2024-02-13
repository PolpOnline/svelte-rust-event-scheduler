use ::entity::{event_user, user};
use chrono::Utc;
use sea_orm::*;

#[derive(Debug)]
pub struct UserToCreate {
    pub name: String,
    pub email: String,
    pub section: i32,
    pub class: String,
    pub admin: bool,
}

pub struct Mutation;

impl Mutation {
    /// Add a user to the database
    pub async fn add_user(db: &DbConn, user: UserToCreate) -> Result<user::Model, DbErr> {
        let user = user::ActiveModel {
            id: Default::default(),
            name: Set(Some(user.name)),
            email: Set(user.email),
            interactive_done: Default::default(),
            section: Set(user.section),
            class: Set(Some(user.class)),
            admin: Set(user.admin),
        };

        user.insert(db).await
    }

    /// Subscribe to events for a user
    pub async fn subscribe_to_events(
        db: &DbConn,
        user_id: i32,
        event_ids: &Vec<i32>,
    ) -> Result<(), DbErr> {
        let mut events = Vec::with_capacity(event_ids.len());

        // Cleanup older event subscriptions for the user
        Self::remove_old_subscriptions_for_user(db, user_id).await?;

        for event_id in event_ids {
            let event_user = event_user::ActiveModel {
                user_id: Set(user_id),
                joined_at: Default::default(),
                left_at: Default::default(),
                event_id: Set(*event_id),
                round: Default::default(),
            };

            events.push(event_user);
        }

        event_user::Entity::insert_many(events)
            .exec(db)
            .await
            .map_err(DbErr::from)?;

        Ok(())
    }

    /// Remove all event subscriptions for a user
    async fn remove_old_subscriptions_for_user(db: &DbConn, user_id: i32) -> Result<(), DbErr> {
        event_user::Entity::delete_many()
            .filter(event_user::Column::UserId.eq(user_id))
            .exec(db)
            .await
            .map_err(DbErr::from)?;

        Ok(())
    }

    /// Join an event (set the joined_at field to now)
    pub async fn join_event(db: &DbConn, user_id: i32, event_id: i32) -> Result<(), DbErr> {
        let event_user: Option<event_user::Model> = event_user::Entity::find()
            .filter(event_user::Column::UserId.eq(user_id))
            .filter(event_user::Column::EventId.eq(event_id))
            .one(db)
            .await?;

        let mut event_user: event_user::ActiveModel = event_user.unwrap().into();

        event_user.joined_at = Set(Some(Utc::now().naive_utc()));

        event_user.update(db).await?;

        Ok(())
    }

    /// Leave an event (set the left_at field to now)
    pub async fn leave_event(db: &DbConn, user_id: i32, event_id: i32) -> Result<(), DbErr> {
        let event_user: Option<event_user::Model> = event_user::Entity::find()
            .filter(event_user::Column::UserId.eq(user_id))
            .filter(event_user::Column::EventId.eq(event_id))
            .one(db)
            .await?;

        let mut event_user: event_user::ActiveModel = event_user.unwrap().into();

        event_user.left_at = Set(Some(Utc::now().naive_utc()));

        event_user.update(db).await?;

        Ok(())
    }
}
