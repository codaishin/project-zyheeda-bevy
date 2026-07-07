use crate::components::map::MapObjectType;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Spawned(pub(crate) MapObjectType);

impl<T> From<T> for Spawned
where
	T: Into<MapObjectType>,
{
	fn from(value: T) -> Self {
		Self(value.into())
	}
}
