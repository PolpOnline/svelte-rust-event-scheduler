//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "event")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: String,
    pub minimum_section: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::event_user::Entity")]
    EventUser,
    #[sea_orm(has_many = "super::round_max_users::Entity")]
    RoundMaxUsers,
}

impl Related<super::event_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventUser.def()
    }
}

impl Related<super::round_max_users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RoundMaxUsers.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::event_user::Relation::User.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::event_user::Relation::Event.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}