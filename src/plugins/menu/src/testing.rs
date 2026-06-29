use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{accessors::get::View, handles_player::PlayerEntity},
};
use std::sync::LazyLock;

pub(crate) static PLAYER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

#[derive(Component)]
#[require(PersistentEntity = *PLAYER)]
pub(crate) struct _Player;

#[derive(SystemParam)]
pub(crate) struct _PlayerParam<'w, 's> {
	players: Query<'w, 's, &'static PersistentEntity, With<_Player>>,
}

impl View<PlayerEntity> for _PlayerParam<'_, '_> {
	fn view(&self) -> Option<PersistentEntity> {
		self.players.single().ok().copied()
	}
}
