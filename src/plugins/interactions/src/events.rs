use bevy::ecs::{entity::Entity, event::Event};

#[derive(Event, Debug, PartialEq)]
pub(crate) struct RayCastEvent {
	pub source: Entity,
	pub target: Entity,
}
