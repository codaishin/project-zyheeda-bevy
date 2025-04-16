use bevy::{ecs::event::Event, math::Vec3};

#[derive(Event, Debug, PartialEq, Clone)]
pub struct MoveInputEvent(pub Vec3);

impl From<Vec3> for MoveInputEvent {
	fn from(translation: Vec3) -> Self {
		Self(translation)
	}
}
