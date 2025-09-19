use crate::traits::accessors::get::Property;
use bevy::ecs::entity::Entity;

impl Property for Entity {
	type TValue<'a> = Self;
}
