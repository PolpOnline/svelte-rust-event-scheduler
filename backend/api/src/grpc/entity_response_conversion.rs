use crate::grpc::event_scheduler::EventsResponse;

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

use entity::event::Model as Event;

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
