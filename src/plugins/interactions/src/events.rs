use bevy::ecs::{entity::Entity, event::Event};

#[derive(Event)]
pub(crate) struct RayCastEvent {
	pub source: Entity,
	pub target: Entity,
}
