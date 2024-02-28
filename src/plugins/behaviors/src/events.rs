use bevy::{ecs::event::Event, math::Vec3};

#[derive(Event, Debug, PartialEq, Clone)]
pub(crate) struct MoveInputEvent(pub Vec3);
