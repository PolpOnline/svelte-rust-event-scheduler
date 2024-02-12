use ::entity::{event, prelude::*};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn get_all_events(db: &DbConn) -> Result<Vec<event::Model>, DbErr> {
        let events = Event::find()
            .order_by(event::Column::Id, Order::Asc)
            .all(db)
            .await?;

        Ok(events)
    }
}
