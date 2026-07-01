use crate::{
	components::persistent_entity::PersistentEntity,
	traits::accessors::get::{View, ViewField},
};
use bevy::{ecs::system::SystemParam, math::InvalidDirectionError};

pub trait HandlesPlayer {
	type TPlayer: for<'w, 's> SystemParam<Item<'w, 's>: View<PlayerEntity>>;
}

pub struct PlayerEntity;

impl ViewField for PlayerEntity {
	type TValue<'a> = Option<PersistentEntity>;
}

#[derive(Debug, PartialEq)]
pub enum DirectionError<TKey> {
	Invalid(InvalidDirectionError),
	KeyHasNoDirection(TKey),
}
