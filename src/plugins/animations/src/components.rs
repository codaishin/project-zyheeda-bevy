pub mod animation_dispatch;

use bevy::ecs::{component::Component, entity::Entity};

#[derive(Component, Debug, PartialEq)]
pub struct Animator {
	pub animation_player: Entity,
}
