pub mod animation_dispatch;

use bevy::ecs::{component::Component, entity::Entity};

#[derive(Component, Default)]
pub struct Animator {
	pub animation_player_id: Option<Entity>,
}
