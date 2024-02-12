use ::entity::event::Model;
use ::entity::prelude::Event;

use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn get_all_events(db: &DbConn) -> Result<Vec<Model>, DbErr> {
        let events = Event::find().all(db).await?;
        Ok(events)
    }
}
