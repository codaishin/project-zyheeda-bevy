use bevy::ecs::{component::Component, entity::Entity};
use common::tools::UnitsPerSecond;

#[derive(Component, Debug, PartialEq)]
pub struct Gravity {
	pub(crate) pull: UnitsPerSecond,
	pub(crate) center: Entity,
}
