use bevy::{ecs::event::Event, math::Vec3};

#[derive(Event, Debug, PartialEq, Clone)]
pub struct MoveClickEvent(pub Vec3);

impl From<Vec3> for MoveClickEvent {
	fn from(translation: Vec3) -> Self {
		Self(translation)
	}
}

#[derive(Event, Debug, PartialEq, Clone)]
pub struct MoveWasdEvent(pub Vec3);

impl From<Vec3> for MoveWasdEvent {
	fn from(translation: Vec3) -> Self {
		Self(translation)
	}
}
