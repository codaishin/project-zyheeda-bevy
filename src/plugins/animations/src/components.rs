use bevy::ecs::{component::Component, entity::Entity};

#[derive(Component, Default)]
pub struct Animator {
	pub animation_player_id: Option<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PlayerMovement {
	Walk,
	Run,
}
