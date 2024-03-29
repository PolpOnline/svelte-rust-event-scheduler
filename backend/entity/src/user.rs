//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: Option<String>,
    #[sea_orm(unique)]
    pub email: String,
    pub interactive_done: bool,
    pub section: i32,
    pub class: Option<String>,
    pub admin: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::event_user::Entity")]
    EventUser,
}

impl Related<super::event_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventUser.def()
    }
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        super::event_user::Relation::Event.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::event_user::Relation::User.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
