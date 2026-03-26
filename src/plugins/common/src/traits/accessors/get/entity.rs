use crate::traits::accessors::get::ViewField;
use bevy::ecs::entity::Entity;

impl ViewField for Entity {
	type TValue<'a> = Self;
}
