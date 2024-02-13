use crate::grpc::event_scheduler::{EventUsersStatusResponse, EventsResponse};
use axum_sessions::async_session::chrono::NaiveDateTime;
use entity::event::Model as Event;
use svelte_rust_event_scheduler_service::EventUserStatus;

macro_rules! impl_from {
    ($src:ident, $dst:ident, $($field:ident),*) => {
        impl From<$src> for $dst {
            fn from(item: $src) -> Self {
                Self {
                    $( $field: item.$field, )*
                }
            }
        }
    };
}

impl_from!(
    Event,
    EventsResponse,
    id,
    name,
    room,
    zone,
    floor,
    minimum_section
);

impl From<EventUserStatus> for EventUsersStatusResponse {
    fn from(item: EventUserStatus) -> Self {
        Self {
            id: item.id,
            name: item.name,
            email: item.email,
            section: item.section,
            class: item.class,
            joined_at: item.joined_at.map(convert_naive_date_time_to_timestamp),
            left_at: item.left_at.map(convert_naive_date_time_to_timestamp),
        }
    }
}

fn convert_naive_date_time_to_timestamp(item: NaiveDateTime) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: item.timestamp(),
        nanos: 0,
    }
}
